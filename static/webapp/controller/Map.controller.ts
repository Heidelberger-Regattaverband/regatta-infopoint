import BaseController from "./Base.controller";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import { map, latLng, tileLayer, MapOptions, Map } from "leaflet";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("map")?.attachMatched((_: Route$MatchedEvent) => this.loadMap(), this);
    // sap.ui.loader.config({ paths: { "osm": "https://www.openlayers.org/api/OpenLayers" } });
    // sap.ui.require(["osm"], () => this.onOsmLoaded());
  }
  // onOsmLoaded() {
  //   alert("OSM loaded");
  // }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onRefreshButtonPress(event: Button$PressEvent): void {

  }
  private loadMap(): void {
    // debugger;
    const options: MapOptions = {
      center: latLng(40.731253, -73.996139),
      zoom: 12,
    };

    const mymap: Map = map("map", options);
    // const mymap: Map = map("map", options);

    tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    }).addTo(mymap);
  }
}