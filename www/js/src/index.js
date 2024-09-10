import { createRoot } from 'react-dom/client';

import App from './App';
import { initializeWasm } from './wasmLoader';

async function startApp() {
    try {
        const wasmModule = await initializeWasm();
        const root = createRoot(document.getElementById('app'));
        root.render(<App wasmModule={wasmModule} />);
    } catch (error) {
        console.error("Failed to start the application:", error);
    }
}
  
startApp();
