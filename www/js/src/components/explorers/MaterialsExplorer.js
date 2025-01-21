import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState([]);

  useEffect(() => {
    if (isLoadingFullData) {
      return;
    }
    materialsService
      .getShelves()
      .then(shelfNames => {
        setShelves(shelfNames || []);
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
        {shelves.map(shelf => <option key={shelf} value={shelf}>{shelf}</option>)}
      </select>
    </div>
  );
};

export default MaterialsExplorer;
