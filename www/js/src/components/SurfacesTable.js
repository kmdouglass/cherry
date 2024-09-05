import { useState } from "react";

import "../css/SurfacesTable.css";

const SurfacesTable = ({ surfaces, setSurfaces }) => {
    const [editingCell, setEditingCell] = useState(null);

    const getSurfaceTypeDefaultValues = (type) => {
        switch (type) {
          case 'Conic':
            return { n: 1.5, thickness: 10, semiDiam: 25, roc: 100 };
          case 'Probe':
            return { n: 1, thickness: 10, semiDiam: 25, roc: "" };
          case 'Stop':
            return { n: 1, thickness: 10, semiDiam: 25, roc: "" };
          default:
            return {};
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

    const handleCellClick = (index, field) => {
        if (index === surfaces.length - 1) return; // Don't allow editing the last row
        setEditingCell({ index, field });
    };
    
    const handleCellChange = (e, index, field) => {
        const newSurfaces = [...surfaces];
        newSurfaces[index][field] = e.target.value;
        setSurfaces(newSurfaces);
    };
    
    const handleCellBlur = () => {
        setEditingCell(null);
    };
    
    const handleKeyDown = (e) => {
        if (e.key === 'Enter') {
          setEditingCell(null);
        }
    };

    const handleInsert = (index) => {
        const newSurfaces = [...surfaces];
        newSurfaces.splice(index + 1, 0, getSurfaceTypeDefaultValues('Conic'));
        setSurfaces(newSurfaces);
    };
  
    const handleDelete = (index) => {
      if (index === 0 || index === surfaces.length - 1) return; // Don't allow deleting Object or Image plane
      const newSurfaces = [...surfaces];
      newSurfaces.splice(index, 1);
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
        if (editingCell && editingCell.index === index && editingCell.field === field) {
        return (
            <div className="editable-cell">
                <span>{value}</span>
                <input
                    type="text"
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
            <div className="editable-cell">
                <span onClick={() => handleCellClick(index, field)}>
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
  
    return (
      <table className="table is-fullwidth">
        <thead>
          <tr>
            <th>Surface type</th>
            <th>Refractive Index</th>
            <th>Thickness</th>
            <th>Semi-Diameter</th>
            <th>Radius of Curvature</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {surfaces.map((surface, index) => (
            <tr key={index}>
              {renderSurfaceTypeCell(surface, index)}
              <td>{renderEditableCell(surface.n, index, "n")}</td>
              <td>{renderEditableCell(surface.thickness, index, "thickness")}</td>
              <td>{renderEditableCell(surface.semiDiam, index, "semiDiam")}</td>
              <td>{renderEditableCell(surface.roc, index, "roc")}</td>
              {renderActionButtons(index)}
            </tr>
          ))}
        </tbody>
      </table>
    );
  };
  
  export default SurfacesTable;
