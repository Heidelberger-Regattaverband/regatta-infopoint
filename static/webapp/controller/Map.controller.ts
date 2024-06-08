import BaseController from "./Base.controller";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import { map, latLng, tileLayer, MapOptions, Map, LatLng, marker, popup, LatLngBounds } from "leaflet";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Button$PressEvent } from "sap/m/Button";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  private readonly participatingClubsModel: JSONModel = new JSONModel();
  private readonly center: LatLng = latLng(49.4093582, 8.694724);
  private map: Map | undefined;
  private bounds: LatLngBounds | undefined;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("map")?.attachMatched((_: Route$MatchedEvent) => {
      this.loadModel().then(() => this.loadMap());
    }, this);
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  onCenterButtonPress(_event: Button$PressEvent): void {
    if (this.map && this.bounds) {
      this.map.fitBounds(this.bounds);
    }
  }

  private async loadModel(): Promise<void> {
    await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${this.getRegattaId()}/participating_clubs`);
  }

  private loadMap(): void {
    const options: MapOptions = {
      center: this.center,
      zoom: 14,
    };

    this.map = map("map", options);

    tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    }).addTo(this.map);

    const pos1: LatLng = latLng(49.41294441086431, 8.690510474742936);
    const pos2: LatLng = latLng(49.41315519733915, 8.691352456928998);
    const pos3: LatLng = latLng(49.41160717484899, 8.678471999709972);
    const pos4: LatLng = latLng(49.41216728849354, 8.692195935777665);
    const pos5: LatLng = latLng(49.41322332864892, 8.700159951566343);
    marker(pos1).bindPopup(popup().setContent("Sattelplatz")).addTo(this.map);
    marker(pos2).bindPopup(popup().setContent("Regattabüro")).addTo(this.map);
    marker(pos3).bindPopup(popup().setContent("Ziel")).addTo(this.map);
    marker(pos4).bindPopup(popup().setContent("Start 1000m")).addTo(this.map);
    marker(pos5).bindPopup(popup().setContent("Start 1500m")).addTo(this.map);

    this.bounds = new LatLngBounds(pos1, pos2);
    this.bounds.extend(pos3).extend(pos4).extend(pos5);
    this.map.fitBounds(this.bounds);

    const data: any[] = this.participatingClubsModel.getData();
    data.forEach((club: any) => {
      // $.ajax({
      //   type: "GET",
      //   url: "https://nominatim.openstreetmap.org/search",
      //   contentType: "application/json",
      //   data: {
      //     format: "json",
      //     city: club.city
      //   },
      //   success: (result: { username: string; }) => {
      //   },
      //   error: (result: any) => {
      //   }
      // });
    });
  }
}