import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Icon from "sap/ui/core/Icon";
import { IconColor } from "sap/ui/core/library";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Monitoring extends BaseController {
  private monitoringModel: JSONModel = new JSONModel();
  private socket?: WebSocket;
  private statusIcon: Icon;

  private units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    super.getRouter()?.getRoute("monitoring")?.attachMatched((_: Route$MatchedEvent) => this.connect(), this);

    super.setViewModel(this.monitoringModel, "monitoring");
    this.statusIcon = this.byId("statusIcon") as Icon;
  }

  onNavBack(): void {
    super.navBack("startpage");
    this.disconnect();
  }

  private updateModel(monitoring: any) {
    const dbConnections = [];
    if (monitoring?.db?.connections) {
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.total"), value: monitoring.db.connections.total });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.used"), value: monitoring.db.connections.used });
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
        cpus.push({ name: cpu.name, value: cpu.usage.toFixed(1) + " %" });
      });
    }

    this.monitoringModel.setProperty("/db", dbConnections);
    this.monitoringModel.setProperty("/mem", mem);
    this.monitoringModel.setProperty("/cpus", cpus);
  }

  private niceBytes(n: number): string {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return (n.toFixed(n < 10 && l > 0 ? 1 : 0) + ' ' + this.units[l]);
  }

  private connect() {
    this.disconnect();

    const location: Location = window.location;
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';

    console.debug('Connecting...');
    this.socket = new WebSocket(`${proto}://${location.host}/api/monitoring`);

    this.socket.onopen = (event: Event) => {
      console.debug('Connected');
      this.statusIcon.setSrc('sap-icon://connected');
      this.statusIcon.setColor(IconColor.Positive);
    }

    this.socket.onmessage = (event: MessageEvent) => {
      const monitoring = JSON.parse(event.data);
      this.updateModel(monitoring);
    }

    this.socket.onclose = () => {
      this.statusIcon.setSrc('sap-icon://disconnected');
      this.statusIcon.setColor(IconColor.Critical);
      console.debug('Disconnected');
      this.socket = undefined;
    }
  }

  private disconnect() {
    if (this.socket) {
      console.debug('Disconnecting...');
      this.socket.close();
      this.socket = undefined;
    }
  }

}