import React from 'react';

const SurfacesModeToggles = ({ appModes, setAppModes }) => {
  return (
    <div className="has-background-light py-2">
      <div className="container">
        <div className="is-flex is-justify-content-center">
          <label className="checkbox">
            <input
              type="checkbox"
              className="mr-2"
              checked={appModes.refractiveIndex}
              onChange={(e) => setAppModes(prev => ({
                ...prev,
                refractiveIndex: e.target.checked
              }))}
            />
            Specify Refractive Index
          </label>
        </div>
      </div>
    </div>
  );
};

export default SurfacesModeToggles;