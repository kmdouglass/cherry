import { useState } from "react";

import "../css/SurfacesTable.css";

const SurfacesTable = () => {
    const [surfaces, setSurfaces] = useState([
      { type: 'Object Plane', n: 1, thickness: 'Infinity', diam: 25, roc: '', actions: 'Insert' },
      { type: 'Conic', n: 1.515, thickness: 5.3, diam: 25, roc: 25.8, actions: 'Insert Delete' },
      { type: 'Conic', n: 1, thickness: 46.6, diam: 25, roc: Infinity, actions: 'Insert Delete' },
      { type: 'Image Plane', n: '', thickness: '', diam: 25, roc: '', actions: '' },
    ]);

    const [editingCell, setEditingCell] = useState(null);

    const getSurfaceTypeDefaultValues = (type) => {
        switch (type) {
          case 'Conic':
            return { n: 1.5, thickness: 10, diam: 25, roc: 100 };
          case 'Probe':
            return { n: 1, thickness: 10, diam: 25, roc: '' };
          case 'Stop':
            return { n: 1, thickness: 10, diam: 25, roc: '' };
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
  
    return (
      <table className="table is-fullwidth">
        <thead>
          <tr>
            <th>Surface type</th>
            <th>n</th>
            <th>thickness</th>
            <th>diam</th>
            <th>roc</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {surfaces.map((surface, index) => (
            <tr key={index}>
              {renderSurfaceTypeCell(surface, index)}
              <td>{renderEditableCell(surface.n, index, "n")}</td>
              <td>{renderEditableCell(surface.thickness, index, "thickness")}</td>
              <td>{renderEditableCell(surface.diam, index, "diam")}</td>
              <td>{renderEditableCell(surface.roc, index, "roc")}</td>
              <td>{surface.actions}</td>
            </tr>
          ))}
        </tbody>
      </table>
    );
  };
  
  export default SurfacesTable;
