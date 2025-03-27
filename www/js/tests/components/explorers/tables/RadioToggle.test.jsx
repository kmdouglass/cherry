import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import RadioToggle from '../../../../src/components/explorers/tables/RadioToggle';

describe('RadioToggle Component', () => {
  const mockOptions = [
    { label: 'Option 1', value: 'option1' },
    { label: 'Option 2', value: 'option2' },
    { label: 'Option 3', value: 'option3' }
  ];
  
  const mockOnChange = vi.fn();

  it('renders all options correctly', () => {
    render(
      <RadioToggle
        options={mockOptions}
        selectedValue="option1"
        onChange={mockOnChange}
        name="testRadioGroup"
      />
    );

    // Check that all options are rendered
    mockOptions.forEach(option => {
      expect(screen.getByText(option.label)).toBeInTheDocument();
    });
  });

  it('checks the correct option based on selectedValue', () => {
    render(
      <RadioToggle
        options={mockOptions}
        selectedValue="option2"
        onChange={mockOnChange}
        name="testRadioGroup"
      />
    );

    // Get all radio inputs
    const radioInputs = screen.getAllByRole('radio');
    
    // Verify the correct one is checked
    expect(radioInputs[1]).toBeChecked();
    expect(radioInputs[0]).not.toBeChecked();
    expect(radioInputs[2]).not.toBeChecked();
  });

  it('calls onChange with correct value when an option is clicked', () => {
    render(
      <RadioToggle
        options={mockOptions}
        selectedValue="option1"
        onChange={mockOnChange}
        name="testRadioGroup"
      />
    );

    // Click on the second option
    fireEvent.click(screen.getByLabelText('Option 2'));
    
    // Check that onChange was called with the correct value
    expect(mockOnChange).toHaveBeenCalledWith('option2');
  });

  it('applies custom className when provided', () => {
    const { container } = render(
      <RadioToggle
        options={mockOptions}
        selectedValue="option1"
        onChange={mockOnChange}
        name="testRadioGroup"
        className="custom-class"
      />
    );

    // Check that the container has the custom class
    const toggleContainer = container.querySelector('.radio-toggle-container');
    expect(toggleContainer).toHaveClass('custom-class');
  });
});
