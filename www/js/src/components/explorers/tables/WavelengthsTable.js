import React, { useState } from 'react';

const WavelengthsTable = ({ wavelengths, setWavelengths, invalidFields, setInvalidFields }) => {
  const [editingCell, setEditingCell] = useState(null);

  const handleCellClick = (value, index) => {
    // Don't allow editing a cell if another cell is invalid
    if (editingCell && invalidFields[editingCell.index]) {
      return;
    }
    setEditingCell({ originalValue: value, index });
  };

  const handleCellChange = (e, index) => {
    const newValue = e.target.value;
    const newWavelengths = [...wavelengths];
    const newInvalidFields = { ...invalidFields };

    // Check if the value is invalid (not a positive number)
    const invalidStates = isNaN(parseFloat(newValue)) || parseFloat(newValue) <= 0;

    if (invalidStates) {
      newInvalidFields[index] = true;
    } else {
      delete newInvalidFields[index];
    }

    newWavelengths[index] = newValue;
    setWavelengths(newWavelengths);
    setInvalidFields(newInvalidFields);
  };

  const handleCellBlur = () => {
    // Do not allow exiting the cell if the input is invalid
    if (invalidFields[editingCell.index]) {
      return;
    }
    setEditingCell(null);
  };

  const handleKeyDown = (e) => {
    if (e.key === 'Enter' && editingCell) {
      // Do not allow exiting the cell if the input is invalid
      if (invalidFields[editingCell.index]) {
        return;
      }
      setEditingCell(null);
    }

    if (e.key === 'Escape' && editingCell) {
      const newWavelengths = [...wavelengths];
      newWavelengths[editingCell.index] = editingCell.originalValue;
      setWavelengths(newWavelengths);
      setInvalidFields({});
      setEditingCell(null);
    }
  };

  const handleInsert = (index) => {
    // Don't allow inserting a row if another cell is invalid
    if (editingCell && invalidFields[editingCell.index]) {
      return;
    }

    const newWavelengths = [...wavelengths];
    newWavelengths.splice(index + 1, 0, 0.5); // Default wavelength value
    setWavelengths(newWavelengths);
  };

  const handleDelete = (index) => {
    // Don't allow deleting the first row
    if (index === 0) return;

    // Don't allow deleting a row if a cell is being edited and invalid
    if (editingCell && invalidFields[editingCell.index]) {
      return;
    }

    const newWavelengths = [...wavelengths];
    newWavelengths.splice(index, 1);
    setWavelengths(newWavelengths);
  };

  const renderEditableCell = (value, index) => {
    const isEditing = editingCell && editingCell.index === index;
    const isInvalid = invalidFields[index];

    if (isEditing) {
      return (
        <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
          <span>{value}</span>
          <input
            type="number"
            value={value}
            onChange={(e) => handleCellChange(e, index)}
            onBlur={handleCellBlur}
            onKeyDown={handleKeyDown}
            autoFocus
            step="any"
            min="0"
          />
        </div>
      );
    }

    return (
      <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
        <span onClick={() => handleCellClick(value, index)}>
          {value}
        </span>
      </div>
    );
  };

  const renderActionButtons = (index) => {
    return (
      <td>
        <div className="action-buttons">
          <button className="button is-small is-primary mr-2" onClick={() => handleInsert(index)}>
            Insert
          </button>
          {index !== 0 && (
            <button className="button is-small is-danger" onClick={() => handleDelete(index)}>
              Delete
            </button>
          )}
        </div>
      </td>
    );
  };

  return (
    <table className="table is-fullwidth">
      <thead>
        <tr>
          <th className="has-text-weight-semibold has-text-right">Wavelength, μm</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {wavelengths.map((wavelength, index) => (
          <tr key={index}>
            <td>{renderEditableCell(wavelength, index)}</td>
            {renderActionButtons(index)}
          </tr>
        ))}
      </tbody>
    </table>
  );
};

export default WavelengthsTable;
