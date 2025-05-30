import { useEffect, useRef, useState } from "react";

import "../css/Navbar.css";

import { EVENT_WORKER_IDLE, EVENT_WORKER_BUSY } from "../services/computeContants";
import SummaryWindow from "./SummaryWindow";
import showAlert from "../modules/alerts";

import bcLensData from "../examples/biconvexLens";
import ccMirrorData from "../examples/concaveMirror";
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
    computeService,
} ) => {
    const [activeDropdown, setActiveDropdown] = useState(null);
    const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false);
    const [isSummaryOpen, setIsSummaryOpen] = useState(false);
    const [workerIdle, setWorkerIdle] = useState(true);
    const fileInputRef = useRef(null);

    // Register the workerIdle state with the compute service
    useEffect(() => {
        const unsubscribe = computeService.subscribe(EVENT_WORKER_IDLE, () => setWorkerIdle(true));
        return () => unsubscribe();
    }, []);

    // Register the workerBusy state with the compute service
    useEffect(() => {
        const unsubscribe = computeService.subscribe(EVENT_WORKER_BUSY, () => setWorkerIdle(false));
        return () => unsubscribe();
    }, []);

    const toggleDropdown = (dropdown) => {
        setActiveDropdown(activeDropdown === dropdown ? null : dropdown);
    };

    const toggleMobileMenu = () => {
        setIsMobileMenuOpen(!isMobileMenuOpen);
    };

    const openDataset = async (inputSurfaces, inputFields, inputAperture, inputWavelengths, inputAppModes) => {
        // Clear materials first
        materialsService.clearSelectedMaterials();

        const newSurfaces = structuredClone(inputSurfaces);
        const newFields = structuredClone(inputFields);
        const newAperture = structuredClone(inputAperture);
        const newWavelengths = structuredClone(inputWavelengths);
        const newAppModes = structuredClone(inputAppModes);

        for (const surface of newSurfaces) {
            // Add any material key to the materials service's selected materials
            if (surface.material) {
                await materialsService.addMaterialToSelectedMaterials(surface.material);
            }

            // Set a default refractive index so that users are not locked out of the RI cell when
            // switching to refractive index mode
            if (!surface.n) {
                surface.n = 1.5;
            }
        }

        setAppModes(newAppModes);

        setSurfaces(newSurfaces);
        setFields(newFields);
        setAperture(newAperture);
        setWavelengths(newWavelengths);
    };

    /**
     * Saves the current dataset to a file.
     * @param {object} data - The dataset to save
     * @param {string} filename - The filename to save the dataset as
     */
    const saveFile = (data, filename) =>{
        // Convert to JSON string
        const jsonString = deepStringify(data);

        // Create a blob and download link
        const blob = new Blob([jsonString], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const link = document.createElement("a");
        link.href = url;
        link.download = filename;

        // Trigger download
        document.body.appendChild(link);
        link.click();

        // Cleanup
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
    }

    const handleSave = () => {
        // Combine data for saving
        const dataToSave = {
            appModes,
            specs: {
                surfaces,
                fields,
                aperture,
                wavelengths
            }
        };

        saveFile(dataToSave, "cherry-specs.json");
    };

    const handleOpen = () => {
        fileInputRef.current?.click();
    }

    const handleExportResults = () => {
        if (!workerIdle) {
            console.debug("Worker is busy, cannot export results");
            return;
        }
        if (!description) {
            console.warn("No results to export");
            return;
        }

        // Combine data for saving
        const dataToSave = {
            paraxialView: description.paraxial_view,
            rayTraceView: computeService.results.data
        };

        saveFile(dataToSave, "cherry-results.json");
    };

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
                    await openDataset(newSurfaces, newFields, newAperture, newWavelengths, jsonData.appModes);
                } else {
                    throw new Error("Invalid file format: missing specific specs data");
                }
            } else {
                throw new Error("Invalid file format: missing specs data and/or appModes");
            }
            
        } catch (error) {
            showAlert(error instanceof Error ? error.message : "Failed to open file");
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
    const handleBiconvexLens = async () => {
        await openDataset(
            bcLensData.surfaces,
            bcLensData.fields,
            bcLensData.aperture,
            bcLensData.wavelengths,
            bcLensData.appModes
        );
    };

    const handleConvexplanoLens = async () => {
        await openDataset(
            cpLensData.surfaces,
            cpLensData.fields,
            cpLensData.aperture,
            cpLensData.wavelengths,
            cpLensData.appModes
        );
    };

    const handleConvexplanoLensWithMaterials = async () => {
        await openDataset(
            cpmLensData.surfaces,
            cpmLensData.fields,
            cpmLensData.aperture,
            cpmLensData.wavelengths,
            cpmLensData.appModes
        );
    };

    const handleConcaveMirror = async () => {
        await openDataset(
            ccMirrorData.surfaces,
            ccMirrorData.fields,
            ccMirrorData.aperture,
            ccMirrorData.wavelengths,
            ccMirrorData.appModes
        );
    };

    const handlePetzvalLens = async () => {
        await openDataset(
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
                            <a className="navbar-item" id="file-open" onClick={handleOpen}>
                                Open...
                            </a>
                            <a
                                className="navbar-item"
                                id="file-export-results"
                                onClick={handleExportResults}
                                style={!workerIdle ? {color: '#999', cursor: 'not-allowed'} : {}}
                                data-tooltip={!workerIdle ? "Cannot export results while computing ray trace." : ""}
                            >
                                Export results
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
                                Convexplano lens / refractive index
                            </a>
                            <a className="navbar-item" id="preset-planoconvex-materials" onClick={handleConvexplanoLensWithMaterials}>
                                Convexplano lens / materials
                            </a>
                            <a className="navbar-item" id="preset-biconvex" onClick={handleBiconvexLens}>
                                Biconvex lens
                            </a>
                            <a className="navbar-item" id="preset-concave-mirror" onClick={handleConcaveMirror}>
                                Concave mirror
                            </a>
                            <a className="navbar-item" id="preset-petzval" onClick={handlePetzvalLens}>
                                Petzval lens
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
                wavelengths={wavelengths}
                appModes={appModes}
                onClose={() => {
                    setIsSummaryOpen(false);
                }}
            />
        </nav>
    );
};

export default Navbar;
