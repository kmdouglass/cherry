import { useEffect, useState } from 'react';

const MaterialsExplorer = ( {materialsService, isLoadingFullData } ) => {
  const [shelves, setShelves] = useState(new Map());
  const [books, setBooks] = useState(new Map());
  const [selectedShelf, setSelectedShelf] = useState(null);

 // Fetch shelf names when the component mounts and all data is loaded
 useEffect(() => {
    if (isLoadingFullData) return;

    materialsService.getShelves()
      .then(shelves => {
        setShelves(shelves || new Map());
        // Set initial shelf
        if (!selectedShelf) {
          const firstShelf = Array.from(shelves?.values() || [])[0];
          setSelectedShelf(firstShelf);
        }
      })
      .catch(error => console.error("Failed to fetch shelf names", error));
  }, [isLoadingFullData, materialsService]);

  // Fetch books when the selected shelf changes
  useEffect(() => {
    if (!selectedShelf) return;

    materialsService.getBooksOnShelf(selectedShelf)
      .then(books => setBooks(books || new Map()))
      .catch(error => console.error("Failed to fetch book names", error));
  }, [selectedShelf, materialsService]);

  const handleShelfChange = (event) => {
    setSelectedShelf(event.target.value);
  }

  const handleBookChange = (event) => {
    console.log(event.target.value);
  }
  
  return (
    <div>
      <h1>Materials Explorer</h1>
      Shelf <select name="shelves" id="shelves" value={selectedShelf || "" } onChange={handleShelfChange}>
        {Array.from(shelves).map(([key, value]) => (
          <option key={key} value={value}>{value}</option>
        ))}
      </select>

      Book <select name="books" id="books" onChange={handleBookChange}>
        {Array.from(books).map(([key, value]) => (
          <option key={key} value={value}>{value}</option>
        ))}
      </select>
    </div>
  );
};

export default MaterialsExplorer;
