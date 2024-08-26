import "./css/cherry.css";

import CutawayView from "./components/CutawayView";
import Navbar from "./components/Navbar";
import TabbedInterface from "./components/TabbedInterface";

function App() {
    // This will be replaced with actual data in the future
    const placeholderData = {};

    return (
        <div className="App">
            <Navbar />
            <div className="container">
                <CutawayView data={placeholderData} />
                <TabbedInterface />
            </div>
        </div>
    );
}

export default App;
