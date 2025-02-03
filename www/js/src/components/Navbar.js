import { useRef, useState } from "react";
import SummaryWindow from "./SummaryWindow";

import cpLensData from "../examples/convexplanoLens";
import cpmLensData from "../examples/convexplanoLensWithMaterials";
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
          const stringKey = (typeof k === "object") ? JSON.stringify(k) : String(k);
          obj[stringKey] = v;
        }
        return {
          dataType: "Map",
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
    description,
    appModes, setAppModes,
    materialsService,
} ) => {
    const [activeDropdown, setActiveDropdown] = useState(null);
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
    const [isSummaryOpen, setIsSummaryOpen] = useState(false);
    const fileInputRef = useRef(null);

    const toggleDropdown = (dropdown) => {
        setActiveDropdown(activeDropdown === dropdown ? null : dropdown);
    };

    const toggleMobileMenu = () => {
        setIsMobileMenuOpen(!isMobileMenuOpen);
    };

    const loadDataset = async (newSurfaces, newFields, newAperture, newWavelengths, newAppModes) => {
        // Clear materials first
        materialsService.clearSelectedMaterials();
        // Use surfaces to add materials
        for (const surface of newSurfaces) {
            if (surface.material) {
                await materialsService.addMaterialToSelectedMaterials(surface.material);
            }
        }

        setAppModes(newAppModes);

        setSurfaces(newSurfaces);
        setFields(newFields);
        setAperture(newAperture);
        setWavelengths(newWavelengths);
    };

    const showAlert = (message) => {
        // Create alert container if it doesn't exist
        let alertContainer = document.getElementById('alert-container');
        if (!alertContainer) {
            alertContainer = document.createElement('div');
            alertContainer.id = 'alert-container';
            alertContainer.style.cssText = `
                position: fixed;
                top: 20px;
                right: 20px;
                z-index: 1000;
                transition: opacity 0.3s ease-in-out;
            `;
            document.body.appendChild(alertContainer);
        }

        // Create new alert element
        const alertElement = document.createElement('div');
        alertElement.style.cssText = `
            background-color: #f44336;
            color: white;
            padding: 15px 20px;
            margin-bottom: 10px;
            border-radius: 4px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.2);
            display: flex;
            justify-content: space-between;
            align-items: center;
            min-width: 300px;
        `;

        // Add message
        const textElement = document.createElement('span');
        textElement.textContent = message;
        alertElement.appendChild(textElement);

        // Add close button
        const closeButton = document.createElement('button');
        closeButton.innerHTML = '&times;';
        closeButton.style.cssText = `
            background: none;
            border: none;
            color: white;
            font-size: 20px;
            cursor: pointer;
            padding: 0 5px;
            margin-left: 10px;
        `;
        closeButton.onclick = () => {
            alertElement.style.opacity = '0';
            setTimeout(() => alertElement.remove(), 300);
        };
        alertElement.appendChild(closeButton);

        // Add to container
        alertContainer.appendChild(alertElement);

        // Auto remove after 5 seconds
        setTimeout(() => {
            if (alertElement.parentElement) {
                alertElement.style.opacity = '0';
                setTimeout(() => {
                    if (alertElement.parentElement) {
                        alertElement.remove();
                    }
                }, 300);
            }
        }, 5000);
    };

    const handleSave = () => {
        if (!description) {
            console.warn("No data to save");
            return;
        }

        // Combine data for saving
        const dataToSave = {
            ...description,  // Preserve any existing description data
            appModes,
            specs: {
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
        fileInputRef.current?.click();
    }

    const handleFileChange = async (event) => {
        const file = event.target.files?.[0];
        if (!file) return;
    
        try {
            // Convert FileReader to Promise-based operation
            const fileContent = await new Promise((resolve, reject) => {
                const reader = new FileReader();
                reader.onload = (e) => resolve(e.target?.result);
                reader.onerror = () => reject(new Error("Failed to read file"));
                reader.readAsText(file);
            });
    
            const jsonData = JSON.parse(fileContent);
            
            // Update all the state with the loaded data from the inputs object
            if (jsonData.specs && jsonData.appModes) {
                const { surfaces: newSurfaces, fields: newFields, aperture: newAperture, wavelengths: newWavelengths } = jsonData.specs;

                if (newSurfaces && newFields && newAperture && newWavelengths) {
                    await loadDataset(newSurfaces, newFields, newAperture, newWavelengths, jsonData.appModes);
                } else {
                    throw new Error("Invalid file format: missing specific specs data");
                }
            } else {
                throw new Error("Invalid file format: missing specs data and/or appModes");
            }
            
        } catch (error) {
            showAlert(error instanceof Error ? error.message : "Failed to load file");
        } finally {
            // Reset the file input so the same file can be selected again
            event.target.value = "";
        }
    };

    // Results handlers
    const handleSummary = () => {
        if (!description) {
            console.warn("No data to summarize");
            return;
        }
        setIsSummaryOpen(true);
    };

    // Examples handlers
    const handleConvexplanoLens = async () => {
        await loadDataset(
            cpLensData.surfaces,
            cpLensData.fields,
            cpLensData.aperture,
            cpLensData.wavelengths,
            cpLensData.appModes
        );
    };

    const handleConvexplanoLensWithMaterials = async () => {
        await loadDataset(
            cpmLensData.surfaces,
            cpmLensData.fields,
            cpmLensData.aperture,
            cpmLensData.wavelengths,
            cpmLensData.appModes
        );
    };

    const handlePetzvalLens = async () => {
        await loadDataset(
            petzvalLensData.surfaces,
            petzvalLensData.fields,
            petzvalLensData.aperture,
            petzvalLensData.wavelengths,
            petzvalLensData.appModes
        );
    };

    return (
        <nav className="navbar" role="navigation" aria-label="main navigation">
            <input
                type="file"
                ref={fileInputRef}
                style={{ display: 'none' }}
                accept=".json"
                onChange={handleFileChange}
            />
            <div className="navbar-brand">
                <a className="navbar-item" href="/cherry">
                    🍒 Cherry Ray Tracer
                </a>
                <button 
                    className={`navbar-burger ${isMobileMenuOpen ? 'is-active' : ''}`}
                    aria-label="menu" 
                    aria-expanded={isMobileMenuOpen}
                    onClick={toggleMobileMenu}
                >
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                    <span aria-hidden="true"></span>
                </button>
            </div>
            <div className={`navbar-menu ${isMobileMenuOpen ? 'is-active' : ''}`}>
                <div className="navbar-start">
                    <div className={`navbar-item has-dropdown ${activeDropdown === "file" ? 'is-active' : ''}`}>
                        <a className="navbar-link is-arrowless" onClick={() => toggleDropdown("file")}>
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

                    <div className={`navbar-item has-dropdown ${activeDropdown === "results" ? 'is-active' : ''}`}>
                        <a className="navbar-link is-arrowless" onClick={() => toggleDropdown("results")}>
                            Results
                        </a>
                        <div className="navbar-dropdown">
                            <a className="navbar-item" id="results-summary" onClick={handleSummary}>
                                Summary
                            </a>
                        </div>
                    </div>

                    <div className={`navbar-item has-dropdown ${activeDropdown === "examples" ? 'is-active' : ''}`}>
                        <a className="navbar-link is-arrowless" onClick={() => toggleDropdown("examples")}>
                            Examples
                        </a>
                        <div className="navbar-dropdown">
                            <a className="navbar-item" id="preset-planoconvex" onClick={handleConvexplanoLens}>
                                Convexplano lens (refractive indexes)
                            </a>
                            <a className="navbar-item" id="preset-planoconvex-materials" onClick={handleConvexplanoLensWithMaterials}>
                                Convexplano lens (materials)
                            </a>
                            <a className="navbar-item" id="preset-petzval" onClick={handlePetzvalLens}>
                                Petzval objective
                            </a>
                        </div>
                    </div>
                </div>

                <div className="navbar-end">
                    <a
                        href="https://github.com/kmdouglass/cherry"
                        className="navbar-item"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        <span className="icon mr-2">
                            <i className="fab fa-github"></i>
                        </span>
                        <span>GitHub</span>
                    </a>
                </div>
            </div>
            <SummaryWindow 
                description={description}
                isOpen={isSummaryOpen}
                onClose={() => {
                    setIsSummaryOpen(false);
                }}
            />
        </nav>
    );
};

export default Navbar;
