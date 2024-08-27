import "./css/cherry.css";

import CutawayView from "./components/CutawayView";
import Navbar from "./components/Navbar";
import DataEntry from "./components/DataEntry";

function App() {
    // This will be replaced with actual data in the future
    const placeholderData = {};

    return (
        <div className="App">
            <Navbar />
            <div className="container">
                <CutawayView data={placeholderData} />
                <DataEntry />
            </div>
        </div>
    );
}

export default App;
