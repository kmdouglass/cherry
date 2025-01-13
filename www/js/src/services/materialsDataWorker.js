import { DATABASE_NAME, MSG_ERR, MSG_FETCH_INITIAL_DATA, MSG_INITIALIZED, OBJECT_STORE_NAME } from './materialsDataConstants';

let db;

onmessage = function (event) {
    console.debug("Received message from the main thread: ", event.data);
  
    const message = event.data[0];
    const arg = event.data[1];

    switch (message) {
        case MSG_FETCH_INITIAL_DATA:
            console.log("Public path: ", __webpack_public_path__);
            fetch(arg)
                .then(response => {
                    if (!response.ok) {
                        self.postMessage([MSG_ERR, `HTTP error! status: ${response.status}`]);
                        // TODO Handle error
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
                            store.add(value, key);
                        }
                    } 
                    req.onerror = e => {
                        self.postMessage([MSG_ERR, e.target.error?.message]);
                    };

                    postMessage(MSG_INITIALIZED);
                });
            break;
    }
};
