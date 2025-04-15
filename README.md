# **Urban Planning GIS App**  
A fast, open-source GIS tool for urban planning, spatial analysis, and geospatial data visualization. This application allows users to select areas, fetch data from OpenStreetMap (Overpass Turbo), and analyze geographic data with a focus on urban development and sustainability.  

## **Features**  
### **🗺️ Map Navigation & Selection**  
- Move around and zoom into maps seamlessly.  
- Select an area using **circle** or **polygon** tools.  
- Center the selection on the screen for easy analysis.  

### **📌 Layer Management**  
- Add, remove, and reorder layers (vector, raster, and real-time data).  
- Fetch OpenStreetMap data dynamically using **Overpass Turbo**.  
- Support for custom map styling (color, opacity, symbology).  

### **📊 Geospatial Analysis** *(Planned)*  
- Query building and road data based on custom filters.  
- Measure areas, distances, and proximity between features.  
- Overlay datasets such as solar potential, pollution, or transit accessibility.  

### **🛠️ Workspace & Project Management**  *(Next working on)*
- Save and load workspaces with selected areas and layers.  
- Export workspaces for future analysis or collaboration.  

### **🔌 Data Integration** *(Planned)*  
- Import/Export **GeoJSON**, **Shapefiles**, and other GIS formats.  
- Support for **WMS/WFS** layers (real-time weather, elevation, etc.).  
- Generate heatmaps and custom visualizations.  

---

## **Installation** *(WIP - Currently for Developers Only)*  
### **Prerequisites**  
- **Rust** (required for compiling the app).  
- Install the following dependencies:  
  ```sh
  sudo apt-get update
  sudo apt-get install --no-install-recommends libasound2-dev libudev-dev
  sudo apt-get install -y libfontconfig1-dev
  sudo apt-get install -y protobuf-compiler
  ```

### **Building from Source**  
```sh
https://github.com/SO9010/map-rs.git
cd map-rs
cargo run
```

---

## **Roadmap**  
- ✅ **Basic Map Navigation & Selection**  
- ✅ **Overpass Turbo Data Fetching**  
- ⏳ **Layer System (WIP)**  
- ⏳ **Attribute Table & Metadata Display**  
- ⏳ **Custom Styling & Visualization**  
- ⏳ **GeoJSON & Shapefile Support**  
- ⏳ **Geospatial Analysis Tools**  

---

## **Contributing**  
Contributions are welcome! Feel free to fork the repository, submit issues, or suggest new features.  

1. Fork the repository.  
2. Create a new branch: `git checkout -b feature-name`  
3. Make changes and commit: `git commit -m "Added new feature"`  
4. Push to your branch: `git push origin feature-name`  
5. Submit a pull request.  

---

## **License**  
[Apache 2.0 License](LICENSE)
