import React, { useState, useEffect, useRef } from 'react';

import {DATABASE_NAME} from "../../services/materialsDataConstants";

const MaterialsNavigator = () => {
  const [searchTerm, setSearchTerm] = useState('');
  const [matches, setMatches] = useState([]);
  const [isDropdownVisible, setIsDropdownVisible] = useState(false);
  const [selectedMaterials, setSelectedMaterials] = useState([]);
  const dropdownRef = useRef(null);

  // Function to search materials in IndexedDB
  const searchMaterials = async (term) => {
    if (!term.trim()) {
      setMatches([]);
      return;
    }

    try {
      const db = await openDB();
      const store = db.transaction('materials', 'readonly').objectStore('materials');
      const allMaterials = await store.getAll();

      console.log('All materials:', allMaterials);

      const matchingMaterials = allMaterials.filter(material => {
        const data = JSON.parse(material.value);
        const searchTermLower = term.toLowerCase();
        return (
          data.book.toLowerCase().includes(searchTermLower) ||
          data.page.toLowerCase().includes(searchTermLower)
        );
      });

      setMatches(matchingMaterials.map(material => {
        const data = JSON.parse(material.value);
        return {
          id: material.id,
          book: data.book,
          page: data.page
        };
      }));
    } catch (error) {
      console.error('Error searching materials:', error);
    }
  };

  // Function to open IndexedDB
  const openDB = () => {
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DATABASE_NAME, 1);
      
      request.onerror = () => reject(request.error);
      request.onsuccess = () => resolve(request.result);
    });
  };

  // Handle search input changes
  const handleSearchChange = (e) => {
    const value = e.target.value;
    setSearchTerm(value);
    searchMaterials(value);
    setIsDropdownVisible(true);
  };

  // Handle material selection
  const handleMaterialSelect = (material) => {
    if (!selectedMaterials.some(m => m.id === material.id)) {
      setSelectedMaterials([...selectedMaterials, material]);
    }
    setSearchTerm('');
    setMatches([]);
    setIsDropdownVisible(false);
  };

  // Handle material removal
  const handleMaterialRemove = (materialId) => {
    setSelectedMaterials(selectedMaterials.filter(m => m.id !== materialId));
  };

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target)) {
        setIsDropdownVisible(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className="container">
      <div className="field">
        <div className="control">
          <input
            className="input"
            type="text"
            placeholder="Search materials..."
            value={searchTerm}
            onChange={handleSearchChange}
          />
        </div>
        
        {isDropdownVisible && matches.length > 0 && (
          <div ref={dropdownRef} className="dropdown is-active">
            <div className="dropdown-menu">
              <div className="dropdown-content">
                {matches.map((material) => (
                  <a
                    key={material.id}
                    className="dropdown-item"
                    onClick={() => handleMaterialSelect(material)}
                  >
                    {material.book} - {material.page}
                  </a>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {selectedMaterials.length > 0 && (
        <div className="box mt-4">
          <h4 className="title is-5">Selected Materials</h4>
          <div className="tags">
            {selectedMaterials.map((material) => (
              <span key={material.id} className="tag is-medium">
                {material.book} - {material.page}
                <button
                  className="delete is-small"
                  onClick={() => handleMaterialRemove(material.id)}
                ></button>
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default MaterialsNavigator;
