import { DATABASE_NAME, MSG_ERR, MSG_CLOSE_DB_CONNECTION, MSG_DB_CLOSED, MSG_FETCH_FULL_DATA, MSG_FETCH_INITIAL_DATA, MSG_FULL_DATA_FETCHED, MSG_INITIALIZED, OBJECT_STORE_NAME } from './materialsDataConstants';

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
                    // Put initial data into indexedDB
                    let req = indexedDB.open(DATABASE_NAME, 1);
                    req.onupgradeneeded = e => {
                        // Create the object store if it doesn't exist
                        db = e.target.result;
                        db.createObjectStore(OBJECT_STORE_NAME);
                    }
                    req.onsuccess = e => {
                        db = e.target.result;

                        let store = db
                            .transaction(DATABASE_NAME, "readwrite")
                            .objectStore(OBJECT_STORE_NAME);

                        for (const [key, value] of Object.entries(data.inner)) {
                            store.put(value, key);
                        }
                    } 
                    req.onerror = e => {
                        throw new Error(`Insertion into materials object store failed: {e.target.error?.message}`);
                    };

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
                        .transaction(DATABASE_NAME, "readwrite")
                        .objectStore(OBJECT_STORE_NAME);

                    for (const [key, value] of Object.entries(data.inner)) {
                        store.put(value, key);
                    }

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
