onmessage = function (event) {
    console.debug("Received message from the main thread:", event.data);

    this.postMessage("Hello from the worker!");
}
