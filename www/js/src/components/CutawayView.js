import { useRef, useEffect } from "react";

import "../css/CutawayView.css";
import { renderCutaway } from "../modules/cutaway";

const CutawayView = ({ description, rawRayPaths }) => { 
    const cutawayRef = useRef(null);

    useEffect(() => {
        if (description && cutawayRef.current) {
            renderCutaway(description, rawRayPaths, cutawayRef.current);
        }
    }, [description]);

    return (
        <div className="cutaway-view" id="cutawayView">
            <svg
                ref={cutawayRef}
                id="cutawaySVG"
                width="100%"
                height="300"
                viewBox="0 0 800 300"
                fill="none"
                stroke="black"
                xmlns="http://www.w3.org/2000/svg">
            </svg>
        </div>
    );
  };
  
  export default CutawayView;
