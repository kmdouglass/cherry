import { useState } from "react";

import "../css/DataEntry.css";
import ApertureTable from "./ApertureTable";
import FieldsTable from "./FieldsTable";
import SurfacesTable from "./SurfacesTable";


const thereAreInvalidFields = (invalidFieldsObj) => {
  // Check that the fields object is not empty.
  // Javascript makes me so sad
  return !(Object.keys(invalidFieldsObj).length === 0) && invalidFieldsObj.constructor === Object;
}

const DataEntry = ({
    surfaces, setSurfaces,
    fields, setFields,
    aperture, setAperture
}) => {
  const [activeTab, setActiveTab] = useState('surfaces');
  const [invalidFields, setInvalidFields] = useState({});

  const handleTabClick = (tab) => {
    // Don't allow switching tabs if another cell is invalid
    if (thereAreInvalidFields(invalidFields)) return;
    setActiveTab(tab);
  }

  const renderTabContent = () => {
    switch(activeTab) {
      case 'surfaces':
        return <SurfacesTable surfaces={surfaces} setSurfaces={setSurfaces} invalidFields={invalidFields} setInvalidFields={setInvalidFields} />;
      case 'fields':
        return <FieldsTable fields={fields} setFields={setFields} invalidFields={invalidFields} setInvalidFields={setInvalidFields} />;
      case 'aperture':
        return <ApertureTable aperture={aperture} setAperture={setAperture} invalidFields={invalidFields} setInvalidFields={setInvalidFields} />;
      default:
        return null;
    }
  };

  return (
    <div className="data-entry">
      <div className="tabs is-centered">
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
        </ul>
      </div>
      <div className="tab-content">
        {renderTabContent()}
      </div>
    </div>
  );
};

export default DataEntry;
