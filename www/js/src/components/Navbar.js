import { useState } from 'react';

import cpLensData from "../examples/convexplanoLens";
import petzvalLensData from "../examples/petzvalLens";

/*
  Converts the nested Maps and Objects to simple JSON strings.
 */
function deepStringify(obj) {
    return JSON.stringify(obj, (key, value) => {
      // Handle Map objects
      if (value instanceof Map) {
        const obj = {};
        for (const [k, v] of value.entries()) {
          // Convert key to string, handling arrays and objects
          const stringKey = (typeof k === 'object') ? JSON.stringify(k) : String(k);
          obj[stringKey] = v;
        }
        return {
          dataType: 'Map',
          value: obj
        };
      }
      
      // Handle Array objects with additional properties
      if (Array.isArray(value)) {
        const plainArray = [...value];
        const additions = Object.entries(value)
          .filter(([key]) => !plainArray.hasOwnProperty(key) && isNaN(parseInt(key)))
          .reduce((obj, [key, val]) => ({ ...obj, [key]: val }), {});
          
        if (Object.keys(additions).length > 0) {
          return { ...additions, elements: plainArray };
        }
        return plainArray;
      }
      
      return value;
    }, 2);
  }

const Navbar = ( {
    surfaces, setSurfaces,
    fields, setFields,
    aperture, setAperture,
    wavelengths, setWavelengths,
    description
} ) => {
    const [activeDropdown, setActiveDropdown] = useState(null);

    const toggleDropdown = (dropdown) => {
        setActiveDropdown(activeDropdown === dropdown ? null : dropdown);
    };

    const handleSave = () => {
        if (!description) {
            console.warn("No data to save");
            return;
        }

        // Combine data for saving
        const dataToSave = {
            ...description,  // Preserve any existing description data
            inputs: {
                surfaces,
                fields,
                aperture,
                wavelengths
            }
        };


        // Convert to JSON string
        const jsonString = deepStringify(dataToSave);

        // Create a blob and download link
        const blob = new Blob([jsonString], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = "cherry.json";

        // Trigger download
        document.body.appendChild(link);
        link.click();

        // Cleanup
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    };

    const handleLoad = () => {
        console.log("Load clicked!");
    }

    const handleConvexplanoLens = () => {
        setSurfaces(cpLensData.surfaces);
        setFields(cpLensData.fields);
        setAperture(cpLensData.aperture);
        setWavelengths(cpLensData.wavelengths);
    };

    const handlePetzvalLens = () => {
        setSurfaces(petzvalLensData.surfaces);
        setFields(petzvalLensData.fields);
        setAperture(petzvalLensData.aperture);
        setWavelengths(petzvalLensData.wavelengths);
    };

    return (
        <nav className="navbar" role="navigation" aria-label="main navigation">
            <div className="navbar-brand">
                <a className="navbar-item" href="/">
                    🍒 Cherry Ray Tracer
                </a>
                <a aria-expanded="false" aria-label="menu" className="navbar-burger" data-target="navMenu" role="button">
                    <span aria-hidden></span>
                    <span aria-hidden></span>
                    <span aria-hidden></span>
                </a>
            </div>
            <div className="navbar-menu">
                <div className="navbar-start">
                    <div className="navbar-item has-dropdown is-hoverable">
                        <a className="navbar-link" onClick={() => toggleDropdown("file")}>
                            File
                        </a>
                        <div className="navbar-dropdown">
                            <a className="navbar-item" id="file-save" onClick={handleSave}>
                                Save
                            </a>
                            <a className="navbar-item" id="file-load" onClick={handleLoad}>
                                Load...
                            </a>
                        </div>
                    </div>

                    <div className="navbar-item has-dropdown is-hoverable">
                        <a className="navbar-link" onClick={() => toggleDropdown("examples")}>
                            Examples
                        </a>
                        <div className="navbar-dropdown">
                            <a className="navbar-item" id="preset-planoconvex" onClick={handleConvexplanoLens}>
                                Convexplano lens
                            </a>
                            <a className="navbar-item" id="preset-petzval" onClick={handlePetzvalLens}>
                                Petzval objective
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    );
};

export default Navbar;
