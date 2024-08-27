import { useState } from "react";

const SurfacesTable = () => {
    const [surfaces, setSurfaces] = useState([
      { type: 'Object Plane', n: 1, thickness: 'Infinity', diam: 25, roc: '', actions: 'Insert' },
      { type: 'Conic', n: 1.515, thickness: 5.3, diam: 25, roc: 25.8, actions: 'Insert Delete' },
      { type: 'Conic', n: 1, thickness: 46.6, diam: 25, roc: Infinity, actions: 'Insert Delete' },
      { type: 'Image Plane', n: '', thickness: '', diam: 25, roc: '', actions: '' },
    ]);

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
              <td>{surface.n}</td>
              <td>{surface.thickness}</td>
              <td>{surface.diam}</td>
              <td>{surface.roc}</td>
              <td>{surface.actions}</td>
            </tr>
          ))}
        </tbody>
      </table>
    );
  };
  
  export default SurfacesTable;
