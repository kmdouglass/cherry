import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState(new Map());

  useEffect(() => {
    if (isLoadingFullData) {
      return;
    }
    materialsService
      .getShelves()
      .then(shelves => {
        console.log(shelves);
        setShelves(shelves || new Map());
      })
      .catch((error) => {
        console.error("Failed to fetch shelf names", error);
      });


      return () => {  }
  }, [isLoadingFullData]);

  const handleShelfChange = (event) => {
    console.log(event.target.value);
  }
  
  return (
    <div>
      <h1>Materials Explorer</h1>
      Shelf <select name="shelves" id="shelves" onChange={handleShelfChange}>
        {Array.from(shelves).map(([key, value]) => (
          <option key={key} value={key}>{value}</option>
        ))}
      </select>
    </div>
  );
};

export default MaterialsExplorer;
