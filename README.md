# **Urban Planning GIS App**  
A fast, open-source GIS tool for urban planning, spatial analysis, and geospatial data visualization. This application allows users to select areas, fetch data from OpenStreetMap (Overpass Turbo), and analyze geographic data with a focus on urban development and sustainability.  

## How to Use the Program

To use the program in its current state:

1. Select the **Selection Tool** (second icon from the left).  
2. Use it to select an area on the map.  
3. On the left side, choose the type of **Overpass request** you want to perform from the dropdown menu.  
4. At the top, select the **Workspace**.  
   - **Note:** Loading may take a few seconds depending on the size of the response.  
5. Once the data is loaded, switch back to the **Cursor Tool** (which also acts as the info tool).  
6. Click on a **house** or other **element**.  
7. You can then change its color based on its properties.

## **Features**  
### **🗺️ Map Navigation & Selection**  
- Move around and zoom into maps seamlessly.  
- Select an area using **circle** or **polygon** tools.  
- Center the selection on the screen for easy analysis.  

### **📌 Layer Management**  
- Add, remove, and reorder layers (vector, raster, and real-time data).  
- Fetch OpenStreetMap data dynamically using **Overpass Turbo**.  

### **🤖 AI-Powered Analysis**
- Interactive chat interface for geospatial queries
- Natural language commands for spatial analysis (nearby features, distances, etc.)
- Automated feature summarization and insights
- Context-aware responses based on current map selection

### **🛠️ Workspace & Project Management**
- Save and load workspaces with selected areas and layers.  
- Persistent workspace state and configuration.
- Request history and data caching.

### **📊 Geospatial Analysis**
- Query building and road data based on custom filters.  
- Measure areas, distances, and proximity between features.  
- Feature counting and spatial statistics.
- Bounding box and polygon-based queries.

### **🔌 Data Integration** *(Planned)*  
- Import/Export **GeoJSON**, **Shapefiles**, and other GIS formats.  
- Support for **WMS/WFS** layers (real-time weather, elevation, etc.).  
- Generate heatmaps and custom visualizations.  

---

## **Building from source** *(WIP - Currently for Developers Only)*  
### **Prerequisites**
1. Rust toolchain (incl. Cargo)
   - Install via https://rustup.rs/

2. Click on your Linux distribution to see the required dependencies:

<details>
  <summary>Fedora / RHEL / CentOS (dnf)</summary>

```bash
sudo dnf install alsa-lib-devel libudev-devel fontconfig-devel protobuf-compiler
```

</details> <details> <summary>Ubuntu / Debian (apt)</summary>

```bash
sudo apt install libasound2-dev libudev-dev libfontconfig1-dev protobuf-compiler
```

</details> <details> <summary>Arch Linux / Manjaro (pacman)</summary>

```bash
sudo pacman -S alsa-lib libudev fontconfig protobuf
```

</details> <details> <summary>Alpine Linux (apk)</summary>

```bash
sudo apk add alsa-lib-dev eudev-dev fontconfig-dev protobuf-dev protobuf
```

</details>

### **Compile project**  
```sh
git clone https://github.com/SO9010/map-rs.git
cd map-rs
cargo build # or 'cargo run', to run the app
```

### **AI Chat Setup (Optional)**
To use the AI-powered chat features:

1. Get an API key from [OpenRouter](https://openrouter.ai/)
2. Edit `src/workspace/ui.rs` and replace the line:
   ```rust
   workspace.llm_agent.set_token("!!! YOUR TOKEN HERE !!!");
   ```
   with your actual API key:
   ```rust
   workspace.llm_agent.set_token("your-actual-api-key-here");
   ```
3. Rebuild the application: `cargo build`

**Note**: The application works without an API key, but AI chat functionality will be unavailable.

---

## **Roadmap**  
- ✅ **Basic Map Navigation & Selection**  
- ✅ **Overpass Turbo Data Fetching**  
- ✅ **AI-Powered Chat & Spatial Analysis**
- ✅ **Workspace Management**
- ⏳ **Layer System (WIP)**  
- ⏳ **Attribute Table & Metadata Display**  
- ⏳ **Custom Styling & Visualization**  
- ⏳ **GeoJSON & Shapefile Support**  
- ⏳ **Advanced Geospatial Analysis Tools**  

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
