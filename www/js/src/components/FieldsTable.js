import { useState } from "react";

import "../css/Table.css";

const FieldsTable = ({ fields, setFields }) => {
  const [editingCell, setEditingCell] = useState(null);
  const [invalidFields, setInvalidFields] = useState({});


  const handleFieldTypeChange = (index, newType) => {
    const newFields = [...fields];
    newFields[index] = { [newType]: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } };
    setFields(newFields);
  };

  const handleCellClick = (index, field) => {
    setEditingCell({ index, field });
  };

  const handleCellChange = (e, index, field) => {
    const newValue = e.target.value;
    const newFields = [...fields];
    const newInvalidFields = { ...invalidFields };

    if (field === 'angle') {
      newFields[index].Angle.angle = newValue;
    } else if (field === 'spacing') {
      newFields[index].Angle.pupil_sampling.SquareGrid.spacing = newValue;
    }

    if (isNaN(parseFloat(newValue))) {
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
  
    setFields(newFields);
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
        <span onClick={() => handleCellClick(index, field)}>
          {value}
        </span>
      </div>
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
