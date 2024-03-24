import MessageToast from "sap/m/MessageToast";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import List from "sap/m/List";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Monitoring extends BaseController {
  private dataLoader: JSONModel;
  private monitoringModel: JSONModel;
  private dbConnectionsList?: List;
  private cpusList?: List;
  private memList?: List;

  private units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];

  onInit(): void {
    const component: MyComponent | undefined = this.getOwnerComponent() as MyComponent | undefined;
    super.getView()?.addStyleClass(component?.getContentDensityClass() || "");

    super.getRouter()?.getRoute("monitoring")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);

    this.monitoringModel = new JSONModel();
    super.setViewModel(this.monitoringModel, "monitoring");

    this.dataLoader = new JSONModel();

    this.dbConnectionsList = this.getView()?.byId("dbConnectionsList") as List;
    this.memList = this.getView()?.byId("memList") as List;
    this.cpusList = this.getView()?.byId("cpusList") as List;
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
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.current"), value: monitoring.db.connections.current });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.idle"), value: monitoring.db.connections.idle });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.created"), value: monitoring.db.connections.created });
    }

    const mem: any[] = [];
    if (monitoring?.sys?.mem) {
      mem.push({ name: this.i18n("monitoring.mem.total"), value: this.niceBytes(monitoring.sys.mem.total) });
      mem.push({ name: this.i18n("monitoring.mem.used"), value: this.niceBytes(monitoring.sys.mem.used) });
      mem.push({ name: this.i18n("monitoring.mem.available"), value: this.niceBytes(monitoring.sys.mem.available) });
      mem.push({ name: this.i18n("monitoring.mem.free"), value: this.niceBytes(monitoring.sys.mem.free) });
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
    this.dbConnectionsList?.setBusy(busy);
    this.memList?.setBusy(busy);
    this.cpusList?.setBusy(busy);
  }

  private niceBytes(n: number): String {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return (n.toFixed(n < 10 && l > 0 ? 1 : 0) + ' ' + this.units[l]);
  }
}