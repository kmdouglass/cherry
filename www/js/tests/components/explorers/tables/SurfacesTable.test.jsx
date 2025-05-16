import React from 'react';
import { describe, it, expect } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import SurfacesTable from '../../../../src/components/explorers/tables/SurfacesTable';

describe('SurfacesTable Component', () => {
  // Mock props needed for the component
  const defaultProps = {
    surfaces: [
      { type: "Object", n: 1, thickness: 0, semiDiam: 12.5, roc: "" }, // Object surface
      { type: "Conic", n: 1.5, thickness: 10, semiDiam: 12.5, roc: 100 }, // Regular surface
      { type: "Image", n: 1, thickness: 0, semiDiam: 12.5, roc: "" }  // Image surface
    ],
    setSurfaces: vi.fn(),
    invalidFields: {},
    setInvalidFields: vi.fn(),
    appModes: { refractiveIndex: true },
    setAppModes: vi.fn(),
    materialsService: {
      selectedMaterials: new Map([
        ['BK7', { page: 'BK7 Glass' }],
        ['SF10', { page: 'SF10 Glass' }]
      ])
    }
  };

  it('renders the RadioToggle component', () => {
    render(<SurfacesTable {...defaultProps} />);
    
    // Check that both options are rendered by getting the labels of the radio buttons
    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(2);
    expect(radios[0]).toHaveAttribute('aria-label', 'Refractive Index');
    expect(radios[1]).toHaveAttribute('aria-label', 'Material');
    
  });
  
  it('shows the refractive index header by default when in refractive index mode', () => {
    render(<SurfacesTable {...defaultProps} />);
    
    // Check that refractive index header is visible
    const tableHeaders = screen.getAllByRole('columnheader');
    expect(tableHeaders.some(header => header.textContent === 'Refractive Index')).toBe(true);
    
    // Check that Material header is not visible (only as option in the toggle)
    expect(tableHeaders.some(header => header.textContent === 'Material')).toBe(false);
  });

  it('shows the material header when in material mode', () => {
    const props = {...defaultProps, appModes: { refractiveIndex: false }};
    render(<SurfacesTable {...props } />);
    
    // Check that Material header is visible
    const tableHeaders = screen.getAllByRole('columnheader');
    expect(tableHeaders.some(header => header.textContent === 'Material')).toBe(true);
    
    // Check that Refractive Index header is not visible
    expect(tableHeaders.some(header => header.textContent === 'Refractive Index')).toBe(false);
  });
  
  it('changes the displayed headers when Material mode is selected', async () => {
    // Create a component with a state that we can control
    const TestComponent = () => {
      const [modes, setModes] = React.useState({ refractiveIndex: true });
      
      return (
        <SurfacesTable
          surfaces={defaultProps.surfaces}
          setSurfaces={defaultProps.setSurfaces}
          invalidFields={defaultProps.invalidFields}
          setInvalidFields={defaultProps.setInvalidFields}
          appModes={modes}
          // for f*ck's sake React state setters can take either functions or values
          setAppModes={(newModes) => setModes(prev => typeof newModes === 'function' ? newModes(prev) : newModes)}
          materialsService={defaultProps.materialsService}
        />
      );
    };
    
    render(<TestComponent />);
    
    // Initially should show Refractive Index in the header
    const tableHeaders = screen.getAllByRole('columnheader');
    expect(tableHeaders[2].textContent).toBe('Refractive Index');
    
    // Click on Material option
    const materialOption = screen.getByLabelText('Material');
    fireEvent.click(materialOption);
    
    // We need to wait for the state update to be processed
    // Using findBy instead of getBy as it's an async query that waits for elements
    const materialHeader = await screen.findByRole('columnheader', { name: /Material/i });
    expect(materialHeader).toBeInTheDocument();
  });

  it('renders the correct number of table rows based on surfaces prop', () => {
    render(<SurfacesTable {...defaultProps} />);
    
    // Get all table rows (excluding header row)
    const rows = screen.getAllByRole('row').slice(1);
    expect(rows).toHaveLength(defaultProps.surfaces.length);
  });

  it('renders different controls for different surface types', () => {
    render(<SurfacesTable {...defaultProps} />);
    
    // The first row should show "Object" as text
    expect(screen.getByText('Object')).toBeInTheDocument();
    
    // The last row should show "Image" as text
    expect(screen.getByText('Image')).toBeInTheDocument();
    
    // The middle row should have a select dropdown for surface type
    const selects = screen.getAllByRole('combobox');
    expect(selects.length).toBeGreaterThan(0);
  });
});
