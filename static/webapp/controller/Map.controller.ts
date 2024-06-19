import BaseController from "./Base.controller";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import { map, latLng, tileLayer, MapOptions, Map, LatLng, marker, popup, LatLngBounds, icon, layerGroup, Marker, TileLayer, LayerGroup, control, latLngBounds, FitBoundsOptions, Control, circle, Circle } from "leaflet";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Button$PressEvent } from "sap/m/Button";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  private readonly participatingClubsModel: JSONModel = new JSONModel();
  private readonly boundsOpts: FitBoundsOptions = { paddingTopLeft: [0, 0], paddingBottomRight: [0, 0] };
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
      this.map.fitBounds(this.bounds, this.boundsOpts);
    }
  }

  private async loadModel(): Promise<void> {
    await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${this.getRegattaId()}/participating_clubs`);
  }

  private loadMap(): void {
    if (!this.map) {
      const layerOsm: TileLayer = tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
      });
      const layerOsmHOT: TileLayer = tileLayer('https://{s}.tile.openstreetmap.fr/hot/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: '© OpenStreetMap contributors, Tiles style by Humanitarian OpenStreetMap Team hosted by OpenStreetMap France'
      });

      const layerRegatta: LayerGroup = this.getRegattaLayerGroup()[0];
      const clubs = this.getClubsLayerGroup();
      const layerClubs: LayerGroup = clubs[0];

      const baseMaps = {
        "OpenStreetMap": layerOsm,
        "OpenStreetMap.HOT": layerOsmHOT
      };
      const overlayMaps = {
        "Regatta": layerRegatta,
        "Vereine": layerClubs,
        "Kreis 250km": this.getCircleLayerGroup()
      };

      const options: MapOptions = {
        doubleClickZoom: true,
        layers: [layerOsm, layerRegatta, layerClubs]
      };
      this.map = map("map", options);
      control.layers(baseMaps, overlayMaps).addTo(this.map);
      control.scale({ imperial: false, metric: true }).addTo(this.map);
      this.bounds = clubs[1];
      this.map.fitBounds(this.bounds, this.boundsOpts);
    }
  }

  private getCircleLayerGroup(): LayerGroup {
    const circle250: Circle = circle(latLng(49.41315519733915, 8.691352456928998), { radius: 250000 });
    return layerGroup([circle250]);
  }

  private getRegattaLayerGroup(): [LayerGroup, LatLngBounds] {
    const pos1: LatLng = latLng(49.41294441086431, 8.690510474742936);
    const posOffice: LatLng = latLng(49.41315519733915, 8.691352456928998);
    const posFinsih: LatLng = latLng(49.41160717484899, 8.678471999709972);
    const posStart1000m: LatLng = latLng(49.41216728849354, 8.692195935777665);
    const posStart1500m: LatLng = latLng(49.41322332864892, 8.700159951566343);
    const mark1: Marker = marker(pos1).bindPopup(popup().setContent("Sattelplatz"));
    const markOffice: Marker = marker(posOffice).bindPopup(popup().setContent("Regattabüro"));
    const markFinish: Marker = marker(posFinsih).bindPopup(popup().setContent("Ziel"));
    const markStart1000m: Marker = marker(posStart1000m).bindPopup(popup().setContent("Start 1000m"));
    const markStart1500m: Marker = marker(posStart1500m).bindPopup(popup().setContent("Start 1500m"));
    const layer: LayerGroup = layerGroup([mark1, markOffice, markFinish, markStart1000m, markStart1500m]);
    const bounds: LatLngBounds = latLngBounds([mark1.getLatLng(), markOffice.getLatLng(), markFinish.getLatLng(), markStart1000m.getLatLng(), markStart1500m.getLatLng()]);
    return [layer, bounds];
  }

  private getClubsLayerGroup(): [LayerGroup, LatLngBounds] {
    const marks: Marker[] = [];
    this.participatingClubsModel.getData().forEach((club: any) => {
      if (club.latitude && club.longitude) {
        const pos: LatLng = latLng(club.latitude, club.longitude);
        const content: string = `<a href="#/clubRegistrations/${club.id}">${club.longName}<br>${club.city}</a>`;
        const mark: Marker = marker(pos).bindPopup(popup().setContent(content));
        if (club.flagUrl) {
          const iconClub = icon({
            iconUrl: club.flagUrl,
            iconSize: [25, 25], // size of the icon
          });
          mark.setIcon(iconClub);
        }
        marks.push(mark);
      }
    });
    return [layerGroup(marks), latLngBounds(marks.map(mark => mark.getLatLng()))];
  }
}