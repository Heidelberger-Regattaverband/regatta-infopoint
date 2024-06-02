import BaseController from "./Base.controller";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

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
  }


}