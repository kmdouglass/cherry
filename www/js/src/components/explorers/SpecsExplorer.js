import { useState } from "react";

import "../../css/DataEntry.css";
import ApertureTable from "./tables/ApertureTable";
import FieldsTable from "./tables/FieldsTable.jsx";
import SurfacesTable from "./tables/SurfacesTable.jsx";
import WavelengthsTable from "./tables/WavelengthsTable";


const thereAreInvalidFields = (invalidFieldsObj) => {
  // Check that the fields object is not empty.
  // Javascript makes me so sad
  return !(Object.keys(invalidFieldsObj).length === 0) && invalidFieldsObj.constructor === Object;
}

const SpecsExplorer = ({
    surfaces, setSurfaces,
    fields, setFields,
    aperture, setAperture,
    wavelengths, setWavelengths,
    invalidFields, setInvalidFields,
    appModes, setAppModes,
    materialsService,
}) => {
  const [activeTab, setActiveTab] = useState('surfaces');

  const handleTabClick = (tab) => {
    // Don't allow switching tabs if another cell is invalid
    if (thereAreInvalidFields(invalidFields)) return;
    setActiveTab(tab);
  }

  const renderTabContent = () => {
    switch(activeTab) {
      case 'surfaces':
        return <SurfacesTable
          surfaces={surfaces} setSurfaces={setSurfaces}
          invalidFields={invalidFields} setInvalidFields={setInvalidFields}
          appModes={appModes} setAppModes={setAppModes}
          materialsService={materialsService}
        />;
      case 'fields':
        return <FieldsTable
          fields={fields} setFields={setFields}
          invalidFields={invalidFields} setInvalidFields={setInvalidFields}
          appModes={appModes} setAppModes={setAppModes}
        />;
      case 'aperture':
        return <ApertureTable aperture={aperture} setAperture={setAperture} invalidFields={invalidFields} setInvalidFields={setInvalidFields} />;
      case 'wavelengths':
        return <WavelengthsTable wavelengths={wavelengths} setWavelengths={setWavelengths} invalidFields={invalidFields} setInvalidFields={setInvalidFields} />;
      default:
        return null;
    }
  };

  return (
    <div className="data-entry">
      <div className="tabs is-centered is-small is-toggle is-toggle-rounded">
        <ul>
          <li className={activeTab === 'surfaces' ? 'is-active' : ''}>
            <a onClick={() => handleTabClick('surfaces')}>Surfaces</a>
          </li>
          <li className={activeTab === 'fields' ? 'is-active' : ''}>
            <a onClick={() => handleTabClick('fields')}>Fields</a>
          </li>
          <li className={activeTab === 'aperture' ? 'is-active' : ''}>
            <a onClick={() => handleTabClick('aperture')}>Aperture</a>
          </li>
          { !appModes.refractiveIndex &&
          <li className={activeTab === 'wavelengths' ? 'is-active' : ''}>
            <a onClick={() => handleTabClick('wavelengths')}>Wavelengths</a>
          </li>}
        </ul>
      </div>
      <div className="tab-content">
        {renderTabContent()}
      </div>
    </div>
  );
};

export default SpecsExplorer;
