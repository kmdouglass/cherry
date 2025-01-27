import { useState } from "react";

import "../../../css/Table.css";

const ApertureTable = ({ aperture, setAperture, invalidFields, setInvalidFields }) => {
    const [editingCell, setEditingCell] = useState(null);

    const handleTypeChange = (e) => {
        // For now we just maintain the same structure since we only have EntrancePupil
        // When we add more types, this handler will need to create the appropriate structure
        const newAperture = {
            [e.target.value]: {
                semi_diameter: aperture.EntrancePupil.semi_diameter
            }
        };
        setAperture(newAperture);
    };

    const handleCellClick = (value) => {
        if (editingCell && invalidFields[editingCell.field]) {
            return;
        }
        setEditingCell({ originalValue: value, field: "semi_diameter" });
    };

    const handleCellChange = (e) => {
        const newValue = e.target.value;
        const newAperture = {
            EntrancePupil: {
                ...aperture.EntrancePupil,
                semi_diameter: newValue
            }
        };
        const newInvalidFields = { ...invalidFields };

        if (isNaN(parseFloat(newValue))) {
            newInvalidFields["semi_diameter"] = true;
        } else {
            delete newInvalidFields["semi_diameter"];
        }

        setAperture(newAperture);
        setInvalidFields(newInvalidFields);
    };

    const handleCellBlur = () => {
        if (invalidFields["semi_diameter"]) {
            return;
        }
        setEditingCell(null);
    };

    const handleKeyDown = (e) => {
        if (e.key === 'Enter') {
            if (invalidFields["semi_diameter"]) {
                return;
            }
            setEditingCell(null);
        }

        if (e.key === 'Escape' && editingCell) {
            const newAperture = {
                EntrancePupil: {
                    ...aperture.EntrancePupil,
                    semi_diameter: editingCell.originalValue
                }
            };

            setAperture(newAperture);
            setInvalidFields({});
            setEditingCell(null);
        }
    };

    const renderEditableCell = (value) => {
        const isEditing = editingCell && editingCell.field === "semi_diameter";
        const isInvalid = invalidFields["semi_diameter"];

        if (isEditing) {
            return (
                <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
                    <span>{value}</span>
                    <input
                        type="number"
                        value={value}
                        min="0"
                        onChange={handleCellChange}
                        onBlur={handleCellBlur}
                        onKeyDown={handleKeyDown}
                        autoFocus
                    />
                </div>
            );
        }
        return (
            <div className={`editable-cell ${isInvalid ? 'invalid' : ''}`}>
                <span onClick={() => handleCellClick(value)}>
                    {value}
                </span>
            </div>
        );
    };

    return (
        <div className="px-4" style={{ maxWidth: '800px' }}>
            <table className="table" style={{ width: '100%' }}>
                <thead>
                    <tr>
                        <th className="has-text-weight-semibold has-text-right" style={{ border: 'none', paddingLeft: 0 }}>
                            Aperture Type
                        </th>
                        <th className="has-text-weight-semibold has-text-right" style={{ border: 'none', paddingLeft: 0 }}>
                            Semi-Diameter
                        </th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td style={{ border: 'none', paddingLeft: 0 }}>
                            <div className="select">
                                <select 
                                    onChange={handleTypeChange}
                                    value="EntrancePupil"
                                >
                                    <option value="EntrancePupil">Entrance Pupil Diameter</option>
                                </select>
                            </div>
                        </td>
                        <td style={{ border: 'none', paddingLeft: 0 }}>
                            {renderEditableCell(aperture.EntrancePupil.semi_diameter)}
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    );
};

export default ApertureTable;
