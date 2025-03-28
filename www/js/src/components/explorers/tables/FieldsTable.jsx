import { useState } from "react";

import "../../../css/Table.css";
import RadioToggle from "./RadioToggle.jsx";

const FieldsTable = ({ fields, setFields, invalidFields, setInvalidFields, appModes, setAppModes }) => {
  const [editingCell, setEditingCell] = useState(null);

  const modeOptions = [
    { label: 'Angle', value: 'Angle' },
    { label: 'Point Source', value: 'PointSource' }
  ];

  /**
   * Returns the default values for a field type.
   * 
   * @param {string} fieldType - The type of field to get the default values for
   * @returns {Object} - The default values for the field type
   */
  const getFieldTypeDefaultValues = (fieldType) => {
    switch (fieldType) {
      case 'Angle':
        return { Angle: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } };
      case 'PointSource':
        return { PointSource: { x: 0, y: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } };
      default:
        console.error(`Unknown field type: ${fieldType}`);
        return {};
    }
  };

  const handleModeChange = (value) => {
    const newFields = [...fields];

    switch (value) {
      case 'Angle':
        setAppModes(prev => ({ ...prev, fieldType: 'Angle' }));

        // Add default values for Angle fields if they don't exist
        for (let field of newFields) {
          if (!field.Angle) {
            field.Angle = { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } };
          }
        }

        break;
      case 'PointSource':
        setAppModes(prev => ({ ...prev, fieldType: 'PointSource' }));

        // Add default values for PointSource fields if they don't exist
        for (let field of newFields) {
          if (!field.PointSource) {
            field.PointSource = { x: 0, y: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } };
          }
        }
        break;
      default:
        console.error(`Unknown mode: ${value}`);
        break;
    }

    setFields(newFields);
  };

  const handleSamplingTypeChange = (index, newType) => {
    const newFields = [...fields];
    newFields[index] = { [newType]: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } };
    setFields(newFields);
  };

  const handleCellClick = (value, index, field) => {
    // Don't allow editing a cell if another cell is invalid
    if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
      return;
    }

    setEditingCell({ originalValue: value, index, field });
  };

  const handleCellChange = (e, index, property) => {
    const newValue = e.target.value;
    const newFields = [...fields];
    const newInvalidFields = { ...invalidFields };

    const invalidStates = (property === "angle" && (newValue < -90.0 || newValue > 90.0))
        || (property === "spacing" && newValue <= 0)
        || isNaN(parseFloat(newValue));

    if (invalidStates) {
        // Invalid input: store the raw input and mark as invalid
        if (!newInvalidFields[index]) {
            newInvalidFields[index] = {};
        }
        newInvalidFields[index][property] = true;
    } else {
        // A valid number; remove any invalid markers
        if (newInvalidFields[index]) {
          delete newInvalidFields[index][property];
          if (Object.keys(newInvalidFields[index]).length === 0) {
              delete newInvalidFields[index];
          }
        }
    }

    if (property === 'angle') {
      newFields[index].Angle.angle = newValue;
    } else if (property === 'y') {
      newFields[index].PointSource.y = newValue;
    } else if (property === 'spacing' && appModes.fieldType === 'Angle') {
      newFields[index].Angle.pupil_sampling.SquareGrid.spacing = newValue;
    } else if (property === 'spacing' && appModes.fieldType === 'PointSource') {
      newFields[index].PointSource.pupil_sampling.SquareGrid.spacing = newValue;
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

    if (e.key === 'Escape' && editingCell) {
      const newFields = [...fields];
      // TODO: Remove the hard-coded "Angle" key as this will be a mess when I add other Field types
      newFields[editingCell.index]["Angle"][editingCell.field] = editingCell.originalValue;

      setFields(newFields);
      setInvalidFields({});
      setEditingCell(null);
    }
  };

  const handleInsert = (index) => {
    // Don't allow inserting a cell if another cell is invalid
    if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
      return;
    }

    const newFields = [...fields];
    if (appModes.fieldType === 'Angle') {
      newFields.splice(index + 1, 0, getFieldTypeDefaultValues('Angle'));
    }
    else if (appModes.fieldType === 'PointSource') {
      newFields.splice(index + 1, 0, getFieldTypeDefaultValues('PointSource'));
    }

    setFields(newFields);
  };

  const handleDelete = (index) => {
    // Don't allow deleting the first row
    if (index === 0) return;

    // Don't allow deleting a cell if a cell is being edited
    if (editingCell && invalidFields[editingCell.index] && invalidFields[editingCell.index][editingCell.field]) {
      return;
    }

    const newFields = [...fields];
    newFields.splice(index, 1);
    setFields(newFields);
  };

  /**
   * Renders a cell in the table that allows the user to select the pupil sampling type.
   * 
   * @param {string} samplingType - The current pupil sampling type 
   * @param {number} index - The row index of the cell in the table
   * @returns {JSX.Element} - The cell element
   */
  const renderSamplingTypeCell = (samplingType, index) => {
    return (
        <td>
            <div className="select">  
                <select
                    value={samplingType}
                    onChange={(e) => handleSamplingTypeChange(index, e.target.value)}
                >
                    <option value="SquareGrid">Square Grid</option>
                </select>
            </div>
        </td>
    );
};

/**
 * Renders a cell in the table that can be edited.
 *
 * @param {number} value - The value of the cell
 * @param {number} index - The row index of the cell in the table
 * @param {string} property - The name of the property in the field object to update
 * @returns {JSX.Element} - The cell element
 */
const renderEditableCell = (value, index, property) => {
    const isEditing = editingCell && editingCell.index === index && editingCell.field === property;
    const isInvalid = invalidFields[index] && invalidFields[index][property];
  
    if (isEditing) {
      return (
        <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
            <span>{value}</span>
            <input
                type="number"
                value={value}
                onChange={(e) => handleCellChange(e, index, property)}
                onBlur={handleCellBlur}
                onKeyDown={handleKeyDown}
                autoFocus
            />
        </div>
      );
    }
    return (
      <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
        <span onClick={() => handleCellClick(value, index, property)}>
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

  const renderComponentDependingOnActiveFieldType = () => {
    const fieldType = appModes.fieldType;
    const fieldTypeHeader = appModes.fieldType === 'Angle' ? 'Angle' : 'Point Source';

    const data = (field) => {
      switch (fieldType) {
        case 'Angle':
          return field.Angle.angle
        case 'PointSource':
          return field.PointSource.y
        default:
          console.error(`Unknown field type: ${fieldType}`);
          return {};
      }
    }

    /**
     * Determines the pupil sampling type for a field.
     * @param {Object} field - The field object 
     * @returns {string} - The pupil sampling type
     */
    const pupilSampling = (field) => {
      switch (fieldType) {
        case 'Angle':
          return Object.keys(field.Angle.pupil_sampling)[0]
        case 'PointSource':
          return Object.keys(field.PointSource.pupil_sampling)[0]
        default:
          console.error(`Unknown field type: ${fieldType}`);
          return {};
      }
    }

    const spacing = (field) => {
      switch (fieldType) {
        case 'Angle':
          return field.Angle.pupil_sampling.SquareGrid.spacing
        case 'PointSource':
          return field.PointSource.pupil_sampling.SquareGrid.spacing
        default:
          console.error(`Unknown field type: ${fieldType}`);
          return {};
      }
    }
    const propertyName = fieldType === 'Angle' ? 'angle' : 'y';

    return (
      <div className="fields-table">
  
        <div className="has-background-light py-2">
            <div className="container">
                <div className="is-flex is-justify-content-center">
                    <RadioToggle
                        options={modeOptions}
                        selectedValue={appModes.fieldType}
                        onChange={handleModeChange}
                        name="fieldType"
                        className="is-flex-direction-row"
                    />
              </div>
            </div>
        </div>
  
        <table className="table is-fullwidth">
          <thead>
            <tr>
              <th className="has-text-weight-semibold has-text-right">{fieldTypeHeader}</th>
              <th className="has-text-weight-semibold has-text-right">Pupil Sampling</th>
              <th className="has-text-weight-semibold has-text-right">Spacing</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {fields.map((field, index) => (
              <tr key={index}>
                <td>{renderEditableCell(data(field), index, propertyName)}</td>
                {renderSamplingTypeCell(pupilSampling(field), index)}
                <td>{renderEditableCell(spacing(field), index, 'spacing')}</td>
                {renderActionButtons(index)}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  };

  return renderComponentDependingOnActiveFieldType();
};

export default FieldsTable;
