import { Circle, FitBoundsOptions, LatLng, LatLngBounds, LayerGroup, Map, MapOptions, Marker, TileLayer, circle, control, icon, latLng, latLngBounds, layerGroup, map, marker, popup, tileLayer } from "leaflet";
import Log from "sap/base/Log";
import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  private readonly participatingClubsModel: JSONModel = new JSONModel();
  private readonly boundsOpts: FitBoundsOptions = { paddingTopLeft: [0, 0], paddingBottomRight: [0, 0] };
  private map?: Map;
  private clubBounds?: LatLngBounds;
  private regattaBounds?: LatLngBounds;
  private centerClubsButton?: Button;
  private centerRegattaButton?: Button;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("map")?.attachMatched((_: Route$MatchedEvent) => {
      // Initialize the map only after the participating-clubs model is loaded.
      // Errors are logged so a network failure does not leave a stuck route
      // and the user can still see the regatta layer (loadMap() is a no-op
      // on subsequent calls because the map is created only once).
      this.loadModel()
        .then(() => this.loadMap())
        .catch((err: unknown) => {
          Log.error("Failed to load map data", err as Error);
          // Still try to render the static regatta layer so the page is not blank.
          this.loadMap();
        });
    }, this);
    this.centerClubsButton = this.byId("centerClubsButton") as Button;
    this.centerRegattaButton = this.byId("centerRegattaButton") as Button;
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  onCenterButtonPress(event: Button$PressEvent): void {
    if (event.getSource() === this.centerClubsButton) {
      this.centerMap(true);
    } else if (event.getSource() === this.centerRegattaButton) {
      this.centerMap(false);
    }
  }

  private centerMap(clubs: boolean): void {
    if (clubs && this.clubBounds?.isValid()) {
      this.map?.fitBounds(this.clubBounds, this.boundsOpts);
    } else if (this.regattaBounds?.isValid()) {
      this.map?.fitBounds(this.regattaBounds, this.boundsOpts);
    }
  }

  private async loadModel(): Promise<void> {
    const regatta: any = await super.getActiveRegatta();
    await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${regatta.id}/clubs`);
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

      const regattaLayer: [LayerGroup, LatLngBounds] = this.getRegattaLayerGroup();
      this.regattaBounds = regattaLayer[1];
      const clubsLayer: [LayerGroup, LatLngBounds] = this.getClubsLayerGroup();
      this.clubBounds = clubsLayer[1];

      const baseMaps = {
        "OpenStreetMap": layerOsm,
        "OpenStreetMap.HOT": layerOsmHOT
      };
      const overlayMaps = {
        "Regatta Orte": regattaLayer[0],
        "Vereine": clubsLayer[0],
        "Entfernung 250km": this.getCircleLayerGroup()
      };
      const options: MapOptions = {
        doubleClickZoom: true,
        layers: [layerOsm, regattaLayer[0], clubsLayer[0]],
      };
      this.map = map("map", options);
      control.layers(baseMaps, overlayMaps).addTo(this.map);
      control.scale({ imperial: false, metric: true }).addTo(this.map);

      this.centerMap(true);
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
        const mark: Marker = marker(pos).bindPopup(popup().setContent(this.buildClubPopup(club)));
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

  /**
   * Builds the popup content for a club marker as a DOM tree (rather than an
   * HTML string) to prevent XSS via `club.longName` / `club.city` (review item 1.1).
   *
   * Leaflet's `Popup.setContent` accepts an `HTMLElement`, which inserts text
   * nodes verbatim — no parser, no script execution.
   */
  private buildClubPopup(club: { id: number | string; longName?: string; city?: string }): HTMLElement {
    const link: HTMLAnchorElement = document.createElement("a");
    // The id is interpolated as a string but only into a hash-route, so it
    // cannot break out of the `href` attribute; still, force a numeric/string
    // representation rather than rich content.
    link.href = `#/clubDetails/${encodeURIComponent(String(club.id))}`;
    link.appendChild(document.createTextNode(club.longName ?? ""));
    link.appendChild(document.createElement("br"));
    link.appendChild(document.createTextNode(club.city ?? ""));
    return link;
  }
}
