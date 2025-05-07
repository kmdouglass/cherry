import { useState } from "react";

import "../../../css/Table.css";

import RadioToggle from "./RadioToggle.jsx";

const SurfacesTable = ({ surfaces, setSurfaces, invalidFields, setInvalidFields, appModes, setAppModes, materialsService }) => {
    const [editingCell, setEditingCell] = useState(null);

    const getSurfaceTypeDefaultValues = (type) => {
        switch (type) {
          case 'Conic':
            return { type: "Conic", n: 1.5, thickness: 10, semiDiam: 12.5, roc: 100 };
          case 'Probe':
            return { type: "Probe", n: 1, thickness: 10, semiDiam: 12.5, roc: "" };
          case 'Stop':
            return { type: "Stop", n: 1, thickness: 10, semiDiam: 12.5, roc: "" };
          default:
            return {};
        }
    };
    
    const modeOptions = [
        { label: 'Refractive Index', value: 'refractiveIndex' },
        { label: 'Material', value: 'material' }
    ];

    const handleModeChange = (value) => {
      switch (value) {
        case 'refractiveIndex':
          setAppModes(prev => ({ ...prev, refractiveIndex: true }));
          break;
        case 'material':
          setAppModes(prev => ({ ...prev, refractiveIndex: false }));
          break;
        default:
          console.error(`Unknown mode: ${value}`);
          break;
      }
    };

    const handleSurfaceTypeChange = (index, newType) => {
        const newSurfaces = [...surfaces];
        const defaultValues = getSurfaceTypeDefaultValues(newType);
        newSurfaces[index] = { 
            ...newSurfaces[index],
            ...defaultValues,
            type: newType,
        };
        setSurfaces(newSurfaces);
    }

    const handleCellClick = (value, index, field) => {
      // Don't allow editing the last row
      if (index === surfaces.length - 1) return;

      // Don't allow editing a cell if another cell is invalid
      if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
          return;
      }

      setEditingCell({ originalValue: value, index, field });
    };
    
    const handleCellChange = (e, index, field) => {
        const newValue = e.target.value;
        const newSurfaces = [...surfaces];
        const newInvalidFields = { ...invalidFields };
        
        newSurfaces[index][field] = newValue;
        if ((newValue != "Infinity") && isNaN(parseFloat(newValue))) {
            // Invalid input: store the raw input and mark as invalid
            if (!newInvalidFields[index]) {
                newInvalidFields[index] = {};
            }
            newInvalidFields[index][field] = true;
        } else {
            // A valid number; remove any invalid markers
            if (newInvalidFields[index]) {
               delete newInvalidFields[index][field];
               if (Object.keys(newInvalidFields[index]).length === 0) {
                   delete newInvalidFields[index];
               }
            }
        }

        // TODO: Use reducer hook?
        setSurfaces(newSurfaces);
        setInvalidFields(newInvalidFields);
    };
    
    const handleCellBlur = () => {
      // Do not allow exiting the cell if the input is invalid
      if (invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
          return;
      }
      setEditingCell(null);
    };

    const handleKeyDown = (e) => {
        if (e.key === 'Enter' && editingCell) {
            // Do not allow exiting the cell if the input is invalid
            if (invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
                return;
            }
            setEditingCell(null);
        }

        if (e.key === 'Escape' && editingCell) {
            const newSurfaces = [...surfaces];
            newSurfaces[editingCell.index][editingCell.field] = editingCell.originalValue;

            // TODO: Use reducer hook?
            setSurfaces(newSurfaces);
            setInvalidFields({});
            setEditingCell(null);
        }
    };

    const handleInsert = (index) => {
      // Don't allow inserting a cell if another cell is invalid
      if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
        return;
      }
  
      const newSurfaces = [...surfaces];
      newSurfaces.splice(index + 1, 0, getSurfaceTypeDefaultValues('Conic'));
      setSurfaces(newSurfaces);
    };
  
    const handleDelete = (index) => {
      // Don't allow deleting Object or Image plane
      if (index === 0 || index === surfaces.length - 1) return;

      // Don't allow deleting a cell if another cell is invalid
      if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
          return;
      }

      const newSurfaces = [...surfaces];
      newSurfaces.splice(index, 1);
      setSurfaces(newSurfaces);
    };

    const handleMaterialChange = (e, index) => {
      const newSurfaces = [...surfaces];

      newSurfaces[index].material = e.target.value;
      setSurfaces(newSurfaces);
    };
  
    const renderSurfaceTypeCell = (surface, index) => {
        if (index === 0) {
          return <td>Object</td>;
        }
        if (index === surfaces.length - 1) {
          return <td>Image</td>;
        }
        return (
          <td>
            <div className="select">
              <select
                value={surface.type}
                onChange={(e) => handleSurfaceTypeChange(index, e.target.value)}
              >
                <option value="Conic">Conic</option>
                <option value="Probe">Probe</option>
                <option value="Stop">Stop</option>
              </select>
            </div>
          </td>
        );
    };

    const renderEditableCell = (value, index, field) => {
        const isEditing = editingCell && editingCell.index === index && editingCell.field === field;
        const isInvalid = invalidFields[index] && invalidFields[index][field];

        if (isEditing) {
          return (
              <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
                  <span>{value}</span>
                  <input
                      type="number"
                      value={value}
                      onChange={(e) => handleCellChange(e, index, field)}
                      onBlur={handleCellBlur}
                      onKeyDown={handleKeyDown}
                      autoFocus
                  />
              </div>
          );
        }
        return (
            <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
                <span onClick={() => handleCellClick(value, index, field)}>
                  {value}
                </span>
            </div>
        );
    };

    const renderActionButtons = (index) => {
      if (index === surfaces.length - 1) return <td><div className="action-buttons"></div></td>;
      return (
        <td>
            <div className="action-buttons">
                <button className="button is-small is-primary mr-2" onClick={() => handleInsert(index)}>Insert</button>
                {index !== 0 && (
                  <button className="button is-small is-danger" onClick={() => handleDelete(index)}>Delete</button>
                )}
            </div>
        </td>
      );
    };

    const renderMaterialHeader = () => {
      if (appModes.refractiveIndex) {
        return <th className="has-text-weight-semibold has-text-right">Refractive Index</th>;
      }
      return <th className="has-text-weight-semibold has-text-right">Material</th>;
    }

    const renderMaterialCell = (value, index, field) => {
      if (appModes.refractiveIndex) {
        return renderEditableCell(value, index, field);
      }

      // If we are at the last (image) surface, don't allow changing the material
      if (index === surfaces.length - 1) {
        return <div></div>;
      }

      // Return a dropdown box with the list of materials
      return (
        <div className="select">
          <select value={surfaces[index].material || ""} onChange={(event) => handleMaterialChange(event, index)}>
            <option value="">Select a material</option>
            {Array.from(materialsService.selectedMaterials).map(([key, material]) => (
              <option key={key} value={key}>{material.page}</option>
            ))}
          </select>
        </div>
      )
    }

    return (
      <div className="surfaces-table">

        <div className="has-background-light py-2">
          <div className="container">
            <div className="is-flex is-justify-content-center">
              <RadioToggle
                options={modeOptions}
                selectedValue={appModes.refractiveIndex ? 'refractiveIndex' : 'material'}
                onChange={handleModeChange}
                name="refactiveIndexOrMaterial"
                className="is-flex-direction-row"
              />
            </div>
          </div>
        </div>

        <div className="table-container">
        <table className="table is-fullwidth is-hoverable">
          <thead>
            <tr>
              <th className="has-text-weight-semibold has-text-right">Surface Type</th>
              { renderMaterialHeader() }
              <th className="has-text-weight-semibold has-text-right">Thickness</th>
              <th className="has-text-weight-semibold has-text-right">Semi-Diameter</th>
              <th className="has-text-weight-semibold has-text-right">Radius of Curvature</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {surfaces.map((surface, index) => (
              <tr key={index}>
                {renderSurfaceTypeCell(surface, index)}

                <td>{renderMaterialCell(surface.n, index, "n")}</td>

                <td>{renderEditableCell(surface.thickness, index, "thickness")}</td>
                <td>{renderEditableCell(surface.semiDiam, index, "semiDiam")}</td>
                <td>{renderEditableCell(surface.roc, index, "roc")}</td>

                {renderActionButtons(index)}
              </tr>
            ))}
          </tbody>
        </table>
        </div>
      </div>
    );
  };
  
  export default SurfacesTable;
