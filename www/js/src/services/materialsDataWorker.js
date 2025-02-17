import {
    INDEX_SHELF_NAME,
    INDEX_SHELF_BOOK_NAME,
    MSG_ERR,
    MSG_CLOSE_DB_CONNECTION,
    MSG_DB_CLOSED,
    MSG_FETCH_FULL_DATA,
    MSG_FETCH_INITIAL_DATA,
    MSG_FULL_DATA_FETCHED,
    MSG_INITIALIZED,
    OBJECT_STORE_NAME
} from './materialsDataConstants';
import { DATABASE_NAME } from "./sharedConstants";

let db;

onmessage = function (event) {
    const message = event.data[0];
    const arg = event.data[1];

    switch (message) {
        // Initialize the database, fetch the initial data, and store it in IndexedDB
        case MSG_FETCH_INITIAL_DATA:
            fetch(arg)
                .then(response => {
                    if (!response.ok) {
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }
                    return response.json()
                })
                .then(data => {
                    // Delete the database if it already exists
                    let req = indexedDB.deleteDatabase(DATABASE_NAME);

                    return new Promise((resolve, reject) => {
                        req.onsuccess = () => {
                            resolve(data);
                        }
                        req.onerror = e => {
                            console.debug(`Failed to delete the database: ${e.target.error?.message}`);
                        }
                    })
                })
                .then(data => {
                    let req = this.indexedDB.open(DATABASE_NAME, 1);

                    req.onupgradeneeded = e => {
                        // Create the object store if it doesn't exist
                        db = e.target.result;
                        const objectStore = db.createObjectStore(OBJECT_STORE_NAME);

                        objectStore.createIndex(INDEX_SHELF_NAME, "shelf", { unique: false });
                        objectStore.createIndex(INDEX_SHELF_BOOK_NAME, ["shelf", "book"], { unique: false });
                    }
                    return new Promise((resolve, reject) => {
                        req.onsuccess = e => {
                            db = e.target.result;
                            resolve([db, data]);
                        }
                        req.onerror = e => {
                            reject(new Error(`Failed initialize to the database: ${e.target.error?.message}`));
                        }
                    })
                })
                .then(([db, data]) => {
                    let store = db
                        .transaction(OBJECT_STORE_NAME, "readwrite")
                        .objectStore(OBJECT_STORE_NAME);

                    for (const [key, value] of Object.entries(data.inner)) {
                        store.put(value, key);
                    }

                    return new Promise((resolve, reject) => {
                        store.transaction.oncomplete = () => {
                            resolve();
                        }
                        store.transaction.onerror = e => {
                            reject(new Error(`Failed to insert data into the object store: ${e.target.error?.message}`));
                        }
                    })
                })
                .then(() => {
                    postMessage(MSG_INITIALIZED);
                })
                .catch(e => {
                    self.postMessage([MSG_ERR, e]);
                });
            break;
       

        // Fetch the full materials data and store it in IndexedDB
        case MSG_FETCH_FULL_DATA:
            fetch(arg)
                .then(response => {
                    if (!response.ok) {
                        console.debug(response);
                        throw new Error(`HTTP error! status: ${response.status}`);
                    }
                    return response.json()
                })
                .then(data => {
                    // Put full data into indexedDB
                    let store = db
                        .transaction(OBJECT_STORE_NAME, "readwrite")
                        .objectStore(OBJECT_STORE_NAME);

                    for (const [key, value] of Object.entries(data.inner)) {
                        store.put(value, key);
                    }

                    return new Promise((resolve, reject) => {
                        store.transaction.oncomplete = () => {
                            resolve();
                        }
                        store.transaction.onerror = e => {
                            reject(new Error(`Failed to insert data into the object store: ${e.target.error?.message}`));
                        }
                    })
                })
                .then(() => {
                    postMessage(MSG_FULL_DATA_FETCHED);
                })
                .catch(e => {
                    self.postMessage([MSG_ERR, e]);
                });
            break;

        // Close the database connection
        case MSG_CLOSE_DB_CONNECTION:
            db.close();
            postMessage(MSG_DB_CLOSED);
            break;
    }
};
