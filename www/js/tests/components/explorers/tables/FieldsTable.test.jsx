import React from 'react';
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import FieldsTable from '../../../../src/components/explorers/tables/FieldsTable';

describe('FieldsTable Component', () => {
  // Mock props needed for the component
  const defaultProps = {
    fields: [
        { Angle: { angle: 0, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } },
        { Angle: { angle: 5, pupil_sampling: { SquareGrid: { spacing: 0.1 } } } },
    ],
    setFields: vi.fn(),
    invalidFields: {},
    setInvalidFields: vi.fn(),
    appModes: { fieldType: "Angle" },
    setAppModes: vi.fn(),
  };

  it('renders the RadioToggle component to switch field types', () => {
    render(<FieldsTable {...defaultProps} />);
    
    // Check that both options are rendered by getting the labels of the radio buttons
    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(2);
    expect(radios[0]).toHaveAttribute('aria-label', 'Angle');
    expect(radios[1]).toHaveAttribute('aria-label', 'Point Source');
    
  });

  it('changes the displayed headers when PointSource mode is selected', async () => {
    const TestComponent = () => {
      const [modes, setModes] = React.useState({ fieldType: "Angle" });
      
      return (
        <FieldsTable
          fields={defaultProps.fields}
          setFields={defaultProps.setFields}
          invalidFields={defaultProps.invalidFields}
          setInvalidFields={defaultProps.setInvalidFields}
          appModes={modes}
          setAppModes={(newModes) => setModes(prev => typeof newModes === 'function' ? newModes(prev) : newModes)}
        />
      );
    };
    
    render(<TestComponent />);
    
    // Initially should show Refractive Index in the header
    const tableHeaders = screen.getAllByRole('columnheader');
    expect(tableHeaders[0].textContent).toBe('Angle');
    
    // Click on Point Source option
    const materialOption = screen.getByLabelText('Point Source');
    fireEvent.click(materialOption);
    
    // We need to wait for the state update to be processed
    // Using findBy instead of getBy as it's an async query that waits for elements
    const pointSourceHeader = await screen.findByRole('columnheader', { name: /Point Source/i });
    expect(pointSourceHeader).toBeInTheDocument();
  });
});
