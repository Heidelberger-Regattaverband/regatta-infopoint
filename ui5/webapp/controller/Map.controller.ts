import {
  Circle,
  FitBoundsOptions,
  LatLng,
  LatLngBounds,
  LayerGroup,
  Map as LeafletMap,
  MapOptions,
  Marker,
  TileLayer,
  circle,
  control,
  icon,
  latLng,
  latLngBounds,
  layerGroup,
  map,
  marker,
  popup,
  tileLayer
} from "leaflet";
import Log from "sap/base/Log";
import MenuItem, { MenuItem$PressEvent } from "sap/m/MenuItem";
import { HTML$AfterRenderingEvent } from "sap/ui/core/HTML";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Routes } from "../model/Routes";
import BaseController from "./Base.controller";

/**
 * Leaflet map of regatta locations and participating clubs, including club-specific route focus.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {
  private readonly participatingClubsModel: JSONModel = new JSONModel([]);
  private readonly boundsOpts: FitBoundsOptions = { paddingTopLeft: [0, 0], paddingBottomRight: [0, 0] };
  private readonly clubMarkers: Map<number, Marker> = new Map();
  private map?: LeafletMap;
  private clubsLayer?: LayerGroup;
  private selectedClubId?: number;
  private clubBounds?: LatLngBounds;
  private regattaBounds?: LatLngBounds;
  private centerClubsButton?: MenuItem;
  private centerRegattaButton?: MenuItem;

  /** Registers route-driven club loading; the rendered HTML container initializes the static map independently. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onAfterShow: this.onAfterShow }, this);

    super.getRouter()?.getRoute(Routes.Map)?.attachMatched(this.onRouteMatched, this);
    super.getRouter()?.getRoute(Routes.MapClub)?.attachMatched(this.onRouteMatched, this);

    this.centerClubsButton = this.byId("centerClubsButton") as MenuItem;
    this.centerRegattaButton = this.byId("centerRegattaButton") as MenuItem;
  }

  /**
   * Captures the current route's focus before asynchronously loading optional club data.
   *
   * @param event - Match for the general map or club-map route; `clubId` is optional.
   */
  private onRouteMatched(event: Route$MatchedEvent): void {
    const args = event.getParameter("arguments") as { clubId?: string } | undefined;
    const clubId = args?.clubId === undefined ? undefined : Number(args.clubId);
    this.selectedClubId = Number.isInteger(clubId) ? clubId : undefined;
    this.updateMapViewport();
    void this.loadModel().catch((err: unknown) => {
      Log.error("Failed to load map data", err as Error);
    });
  }

  /**
   * Initializes Leaflet only after its native container exists, even if club loading has failed or is still pending.
   *
   * @param event - Rendering event from the HTML container, including preserved-DOM rerenders.
   */
  onMapAfterRendering(event: HTML$AfterRenderingEvent): void {
    if (super.getView()?.isDestroyed() !== false) {
      return;
    }
    const container = event.getSource().getDomRef() as HTMLElement | null;
    if (container) {
      this.loadMap(container);
    }
  }

  /** Recalculates map size and bounds after a cached view becomes visible again. */
  private onAfterShow(): void {
    this.map?.invalidateSize({ pan: false });
    this.updateMapViewport();
  }

  /** Unsubscribes route handlers and releases Leaflet's DOM/window listeners on view destruction. */
  onExit(): void {
    for (const route of [Routes.Map, Routes.MapClub]) {
      super.getRouter()?.getRoute(route)?.detachMatched(this.onRouteMatched, this);
    }
    this.map?.remove();
    this.map = undefined;
    this.clubsLayer = undefined;
    this.clubMarkers.clear();
    super.onExit?.();
  }

  onNavBack(): void {
    super.navBack(Routes.StartPage);
  }

  /**
   * Selects the club or regatta viewport according to which center button was pressed.
   *
   * @param event - Press event whose source identifies the requested viewport.
   */
  onCenterButtonPress(event: MenuItem$PressEvent): void {
    if (event.getSource() === this.centerClubsButton) {
      this.centerClubsMap();
    } else if (event.getSource() === this.centerRegattaButton) {
      this.centerRegattaMap();
    }
  }

  /** Fits all club markers, falling back to regatta bounds if no valid club bounds exist. */
  private centerClubsMap(): void {
    if (this.clubBounds?.isValid()) {
      this.map?.fitBounds(this.clubBounds, this.boundsOpts);
    } else if (this.regattaBounds?.isValid()) {
      this.map?.fitBounds(this.regattaBounds, this.boundsOpts);
    }
  }

  private centerRegattaMap(): void {
    if (this.regattaBounds?.isValid()) {
      this.map?.fitBounds(this.regattaBounds, this.boundsOpts);
    }
  }

  /**
   * Loads participating clubs and refreshes the existing layer only after a successful request.
   * Fixed regatta locations remain available on failure; data arriving before rendering is used at initialization.
   *
   * @returns Whether club loading succeeded for a still-live view; active-regatta lookup failures reject.
   */
  private async loadModel(): Promise<boolean> {
    const view = super.getView();
    const regatta: any = await super.getActiveRegatta();
    if (view?.isDestroyed() !== false) {
      return false;
    }
    const succeeded = await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${regatta.id}/clubs`);
    if (succeeded && !view.isDestroyed()) {
      this.refreshClubsLayer();
      this.updateMapViewport();
      return true;
    }
    return false;
  }

  /**
   * Creates fixed and optional layers once per native container; replacing the container releases the previous map.
   *
   * @param container - Rendered HTML element owned by this view, not a globally resolved DOM ID.
   */
  private loadMap(container: HTMLElement): void {
    if (this.map && this.map.getContainer() !== container) {
      this.map.remove();
      this.map = undefined;
    }
    if (!this.map) {
      const layerOsm: TileLayer = tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
        maxZoom: 19,
        attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
      });
      const layerOsmHOT: TileLayer = tileLayer("https://{s}.tile.openstreetmap.fr/hot/{z}/{x}/{y}.png", {
        maxZoom: 19,
        attribution:
          "© OpenStreetMap contributors, Tiles style by Humanitarian OpenStreetMap Team hosted by OpenStreetMap France"
      });

      const regattaLayer: [LayerGroup, LatLngBounds] = this.getRegattaLayerGroup();
      this.regattaBounds = regattaLayer[1];
      const clubsLayer: [LayerGroup, LatLngBounds] = this.getClubsLayerGroup();
      this.clubsLayer = clubsLayer[0];
      this.clubBounds = clubsLayer[1];

      const baseMaps = {
        OpenStreetMap: layerOsm,
        "OpenStreetMap.HOT": layerOsmHOT
      };
      const overlayMaps = {
        "Regatta Orte": regattaLayer[0],
        Vereine: clubsLayer[0],
        "Entfernung 250km": this.getCircleLayerGroup()
      };
      const options: MapOptions = {
        doubleClickZoom: true,
        layers: [layerOsm, regattaLayer[0], clubsLayer[0]]
      };
      this.map = map(container, options);
      control.layers(baseMaps, overlayMaps).addTo(this.map);
      control.scale({ imperial: false, metric: true }).addTo(this.map);

      this.updateMapViewport();
    }
  }

  /** Updates markers and bounds without replacing the overlay registered in Leaflet's layer selector. */
  private refreshClubsLayer(): void {
    if (!this.clubsLayer || !this.map) {
      return;
    }
    const [layer, bounds] = this.getClubsLayerGroup();
    this.clubBounds = bounds;
    this.clubsLayer.clearLayers();
    layer.eachLayer((marker) => this.clubsLayer?.addLayer(marker));
  }

  /** Focuses the current route's club if present; otherwise fits clubs or falls back to regatta locations. */
  private updateMapViewport(): void {
    if (this.selectedClubId !== undefined && this.clubMarkers.has(this.selectedClubId)) {
      this.panToClub(this.selectedClubId);
    } else {
      this.centerClubsMap();
    }
  }

  /**
   * Zooms to a known club marker and opens its popup; unknown club IDs are ignored.
   *
   * @param clubId - Backend club ID used to look up the marker created for the participating club.
   */
  private panToClub(clubId: number): void {
    const mark = this.clubMarkers.get(clubId);
    if (mark) {
      this.map?.setView(mark.getLatLng(), 13);
      mark.openPopup();
    }
  }

  /**
   * Creates the optional 250 km distance overlay around the regatta office.
   *
   * @returns A Leaflet layer group containing the radius circle.
   */
  private getCircleLayerGroup(): LayerGroup {
    const circle250: Circle = circle(latLng(49.41315519733915, 8.691352456928998), { radius: 250000 });
    return layerGroup([circle250]);
  }

  /**
   * Builds the fixed regatta-location markers.
   *
   * @returns The marker layer and its viewport bounds, in that order.
   */
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
    const bounds: LatLngBounds = latLngBounds([
      mark1.getLatLng(),
      markOffice.getLatLng(),
      markFinish.getLatLng(),
      markStart1000m.getLatLng(),
      markStart1500m.getLatLng()
    ]);
    return [layer, bounds];
  }

  /**
   * Builds markers for clubs with numeric or decimal-string coordinates and indexes them by ID for route-based focus.
   *
   * @returns The marker layer and its viewport bounds, in that order.
   */
  private getClubsLayerGroup(): [LayerGroup, LatLngBounds] {
    const marks: Marker[] = [];
    this.clubMarkers.clear();
    const data: unknown = this.participatingClubsModel.getData();
    const clubs = Array.isArray(data) ? data : [];
    clubs.forEach((club: any) => {
      const latitude = this.parseCoordinate(club?.latitude);
      const longitude = this.parseCoordinate(club?.longitude);
      if (
        club?.id != null &&
        Number.isInteger(Number(club.id)) &&
        Number.isFinite(latitude) &&
        Number.isFinite(longitude) &&
        Math.abs(latitude) <= 90 &&
        Math.abs(longitude) <= 180
      ) {
        const pos: LatLng = latLng(latitude, longitude);
        const mark: Marker = marker(pos).bindPopup(popup().setContent(this.buildClubPopup(club)));
        if (club.flagUrl) {
          const iconClub = icon({
            iconUrl: club.flagUrl,
            iconSize: [25, 25] // size of the icon
          });
          mark.setIcon(iconClub);
        }
        this.clubMarkers.set(Number(club.id), mark);
        marks.push(mark);
      }
    });
    return [layerGroup(marks), latLngBounds(marks.map((mark) => mark.getLatLng()))];
  }

  /**
   * Normalizes coordinates, including the decimal strings serialized by the backend, without coercing absent values to zero.
   *
   * @param value - Coordinate from the club response; only numbers and nonempty numeric strings are accepted.
   * @returns The numeric coordinate, or `NaN` for unsupported or nonnumeric values; callers validate range and finiteness.
   */
  private parseCoordinate(value: unknown): number {
    if (typeof value === "number") {
      return value;
    }
    return typeof value === "string" && value.trim() !== "" ? Number(value) : Number.NaN;
  }

  /**
   * Builds the popup content for a club marker as a DOM tree (rather than an
   * HTML string) so club names and cities are inserted as text rather than parsed as markup.
   *
   * Leaflet's `Popup.setContent` accepts an `HTMLElement`, which inserts text
   * nodes verbatim. The club ID is URL-encoded in the detail link.
   *
   * @param club - Club ID, long name, and city used to build the text-only linked popup.
   * @returns An anchor element linking to club details, with name and city separated by a line break.
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
