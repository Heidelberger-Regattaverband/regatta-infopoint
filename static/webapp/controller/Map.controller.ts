import BaseController from "./Base.controller";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import { map, latLng, tileLayer, MapOptions, Map } from "leaflet";
import JSONModel from "sap/ui/model/json/JSONModel";
import * as $ from "jquery";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MapController extends BaseController {

  private readonly participatingClubsModel: JSONModel = new JSONModel();

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getRouter()?.getRoute("map")?.attachMatched((_: Route$MatchedEvent) => {
      this.loadModel().then(() => this.loadMap());
    }, this);
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  async loadModel(): Promise<void> {
    await super.updateJSONModel(this.participatingClubsModel, `/api/regattas/${this.getRegattaId()}/participating_clubs`);
  }

  private loadMap(): void {
    // debugger;
    const options: MapOptions = {
      center: latLng(49.4093582, 8.694724),
      zoom: 14,
    };

    const mymap: Map = map("map", options);
    // const mymap: Map = map("map", options);

    tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    }).addTo(mymap);

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