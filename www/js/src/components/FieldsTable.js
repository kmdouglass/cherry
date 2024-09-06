import { useState } from "react";

import "../css/Table.css";

const FieldsTable = ({ fields, setFields }) => {
  const [editingCell, setEditingCell] = useState(null);

  const handleFieldTypeChange = (index, newType) => {
    const newFields = [...fields];
    newFields[index] = { [newType]: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } };
    setFields(newFields);
  };

  const handleCellClick = (index, field) => {
    setEditingCell({ index, field });
  };

  const handleCellChange = (e, index, field) => {
    const newFields = [...fields];
    if (field === 'angle') {
      newFields[index].Angle.angle = parseFloat(e.target.value);
    } else if (field === 'spacing') {
      newFields[index].Angle.pupil_sampling.SquareGrid.spacing = parseFloat(e.target.value);
    }
    setFields(newFields);
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
    const newFields = [...fields];
    newFields.splice(index + 1, 0, { Angle: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } });
    setFields(newFields);
  };

  const handleDelete = (index) => {
    if (index === 0) return; // Don't allow deleting the first row
    const newFields = [...fields];
    newFields.splice(index, 1);
    setFields(newFields);
  };

  const renderFieldTypeCell = (fieldType, index) => {
        return (
            <td>
                <div className="select">  
                    <select
                        value={fieldType}
                        onChange={(e) => handleFieldTypeChange(index, e.target.value)}
                    >
                        <option value="Angle">Angle</option>
                    </select>
                </div>
            </td>
        );
  };

  const renderSamplingTypeCell = (fieldType, index) => {
    return (
        <td>
            <div className="select">  
                <select
                    value={fieldType}
                    onChange={(e) => handleFieldTypeChange(index, e.target.value)}
                >
                    <option value="SquareGrid">Square Grid</option>
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
      <span onClick={() => handleCellClick(index, field)} style={{ cursor: 'pointer' }}>
        {value}
      </span>
    );
  };

  const renderActionButtons = (index) => {
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
          <th>Field Type</th>
          <th>Angle</th>
          <th>Pupil Sampling</th>
          <th>Spacing</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {fields.map((field, index) => (
          <tr key={index}>
            {renderFieldTypeCell(Object.keys(field)[0], index)}
            <td>{renderEditableCell(field.Angle.angle, index, 'angle')}</td>
            {renderSamplingTypeCell(Object.keys(field.Angle.pupil_sampling)[0], index)}
            <td>{renderEditableCell(field.Angle.pupil_sampling.SquareGrid.spacing, index, 'spacing')}</td>
            {renderActionButtons(index)}
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default FieldsTable;
