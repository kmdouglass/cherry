import React from "react";
import "../../../css/RadioToggle.css";

/**
 * Type definition for a single option in the RadioToggle component.
 * 
 * @typedef {Object} Option
 * @property {string} label - Display text for the option
 * @property {string} value - Value of the option
 */

/** 
 * A component of radio buttons for selecting a single value from a set of options.
 * 
 * @param {Object} props
 * @param {Array<Option>} props.options - Array of option objects with label and value properties
 * @param {string} props.selectedValue - Currently selected value
 * @param {Function} props.onChange - Function called when selection changes
 * @param {string} props.name - Name for the radio button group
 * @param {string} [props.className] - Optional CSS class
 */
const RadioToggle = ({ options, selectedValue, onChange, name, className }) => {
    return (
        <div className={`radio-toggle-container ${className || ''}`}>
            {options.map((option) => (
                <label key={option.value} className="radio-toggle-label">
                <input
                    type="radio"
                    name={name}
                    value={option.value}
                    checked={selectedValue === option.value}
                    onChange={() => onChange(option.value)}
                    className="radio-toggle-input"
                    aria-label={option.label}
                />
                <span className="radio-toggle-text">{option.label}</span>
                </label>
            ))}
        </div>
    );
}

export default RadioToggle;
