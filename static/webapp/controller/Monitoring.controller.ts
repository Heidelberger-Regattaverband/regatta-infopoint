import MessageToast from "sap/m/MessageToast";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import Control from "sap/ui/core/Control";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Monitoring extends BaseController {
  private dataLoader: JSONModel;
  private monitoringModel: JSONModel;
  private racesList?: Control;
  private heatsList?: Control;
  private registrationsList?: Control;
  private athletesList?: Control;

  private units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    super.getRouter()?.getRoute("monitoring")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);

    this.monitoringModel = new JSONModel();
    super.setViewModel(this.monitoringModel, "monitoring");

    this.dataLoader = new JSONModel();

    this.registrationsList = this.getView()?.byId("registrationsList") as Control;
    this.racesList = this.getView()?.byId("racesList") as Control;
    this.heatsList = this.getView()?.byId("heatsList") as Control;
    this.athletesList = this.getView()?.byId("athletesList") as Control;
  }

  onNavBack(): void {
    super.navBack("startpage");
  }

  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    await this.loadStatistics();
  }

  private async loadStatistics(): Promise<void> {
    this.setBusy(true);
    let monitoring: any;

    // load statistic data from backend
    if (await super.updateJSONModel(this.dataLoader, `/api/monitoring`)) {
      monitoring = this.dataLoader.getData();
      MessageToast.show(super.i18n("msg.dataUpdated"));
    } else {
      monitoring = {};
    }

    // transform monitoring data into human readable format
    const dbConnections = [];
    if (monitoring?.db?.connections) {
      dbConnections.push({ name: "Aktuell", value: monitoring.db.connections.current });
      dbConnections.push({ name: "Idle", value: monitoring.db.connections.idle });
      dbConnections.push({ name: "Erzeugt", value: monitoring.db.connections.created });
    }

    const mem: any[] = [];
    if (monitoring?.sys?.mem) {
      mem.push({ name: "Insgesamt", value: this.niceBytes(monitoring.sys.mem.total) });
      mem.push({ name: "Benutzt", value: this.niceBytes(monitoring.sys.mem.used) });
      mem.push({ name: "Frei", value: this.niceBytes(monitoring.sys.mem.free) });
      mem.push({ name: "Verfügbar", value: this.niceBytes(monitoring.sys.mem.available) });
    }

    const cpus: any[] = [];
    if (monitoring?.sys?.cpus) {
      monitoring.sys.cpus.forEach((cpu: any, index: number) => {
        cpus.push({ name: cpu.name, value: cpu.usage });
      });
    }

    this.monitoringModel.setProperty("/db", dbConnections);
    this.monitoringModel.setProperty("/mem", mem);
    this.monitoringModel.setProperty("/cpus", cpus);

    this.setBusy(false);
  }

  private setBusy(busy: boolean): void {
    this.registrationsList?.setBusy(busy);
    this.racesList?.setBusy(busy);
    this.heatsList?.setBusy(busy);
    this.athletesList?.setBusy(busy);
  }

  private niceBytes(n: number): String {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return (n.toFixed(n < 10 && l > 0 ? 1 : 0) + ' ' + this.units[l]);
  }
}