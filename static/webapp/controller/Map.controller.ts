import BaseController from "./Base.controller";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import { map, latLng, tileLayer, MapOptions } from "leaflet";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("map")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);
    sap.ui.loader.config({ paths: { "osm": "https://www.openlayers.org/api/OpenLayers" } });
    sap.ui.require(["osm"], () => this.onOsmLoaded());
  }
  onOsmLoaded() {
    alert("OSM loaded");
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    await this.loadStatistics();
  }

  private async loadStatistics(): Promise<void> {
    const options: MapOptions = {
      center: latLng(40.731253, -73.996139),
      zoom: 12,
    };
    
    const mymap = map('map', options);
    
    const key = "YOUR_MAPTILER_API_KEY_HERE";
    
    tileLayer(`https://api.maptiler.com/maps/streets-v2/{z}/{x}/{y}.png?key=${key}`,{ //style URL
      tileSize: 512,
      zoomOffset: -1,
      minZoom: 1,
      attribution: "\u003ca href=\"https://www.maptiler.com/copyright/\" target=\"_blank\"\u003e\u0026copy; MapTiler\u003c/a\u003e \u003ca href=\"https://www.openstreetmap.org/copyright\" target=\"_blank\"\u003e\u0026copy; OpenStreetMap contributors\u003c/a\u003e",
      crossOrigin: true
    }).addTo(mymap);
  }
}