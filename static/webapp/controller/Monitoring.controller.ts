import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Button from "sap/m/Button";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class MonitoringController extends BaseController {

  private readonly units = ['bytes', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
  private readonly monitoringModel: JSONModel = new JSONModel();
  private socket?: WebSocket;
  private statusButton: Button;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeHide: this.onBeforeHide }, this);
    super.getRouter()?.getRoute("monitoring")?.attachMatched((_: Route$MatchedEvent) => this.connect(), this);
    super.setViewModel(this.monitoringModel, "monitoring");
    this.statusButton = this.byId("statusButton") as Button;
  }

  private onBeforeHide(): void {
    this.disconnect();
  }

  onNavBack(): void {
    super.navToStartPage();
    this.disconnect();
    this.updateModel({});
  }

  onStatusButtonPress(): void {
    this.connect();
  }

  private updateModel(monitoring: any) {
    const dbConnections = [];
    if (monitoring?.db?.connections) {
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.total"), value: monitoring.db.connections.total });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.used"), value: monitoring.db.connections.used });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.idle"), value: monitoring.db.connections.idle });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.created"), value: monitoring.db.connections.created });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.closedIdleTimeout"), value: monitoring.db.connections.closedIdleTimeout });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.closedMaxLifetime"), value: monitoring.db.connections.closedMaxLifetime });
      dbConnections.push({ name: this.i18n("monitoring.dbConnections.closedError"), value: monitoring.db.connections.closedError });
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
      monitoring.sys.cpus.forEach((cpu: any, _index: number) => {
        cpus.push({ name: cpu.name, value: cpu.usage.toFixed(1) + " %" });
      });
    }

    const sys: any[] = [];
    if (monitoring?.sys?.uptime) {
      sys.push({ name: this.i18n("monitoring.sys.uptime"), value: this.niceDuration(monitoring.sys.uptime.secs) });
    }

    const app: any[] = [];
    if (monitoring?.app) {
      app.push({ name: this.i18n("monitoring.app.mem_current"), value: this.niceBytes(monitoring.app.mem_current) });
      app.push({ name: this.i18n("monitoring.app.mem_peak"), value: this.niceBytes(monitoring.app.mem_peak) });
    }

    this.monitoringModel.setProperty("/db", dbConnections);
    this.monitoringModel.setProperty("/mem", mem);
    this.monitoringModel.setProperty("/cpus", cpus);
    this.monitoringModel.setProperty("/sys", sys);
    this.monitoringModel.setProperty("/app", app);
  }

  private niceBytes(n: number): string {
    let l: number = 0;
    while (n >= 1024 && ++l) {
      n = n / 1024;
    }
    return (n.toFixed(n < 10 && l > 0 ? 1 : 0) + ' ' + this.units[l]);
  }

  private niceDuration(seconds: number): string {
    const duration = new Date(seconds * 1000).toISOString().slice(11, 19);
    return duration;
  }

  private connect() {
    this.disconnect();

    const location: Location = window.location;
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';

    console.debug('Connecting...');
    this.socket = new WebSocket(`${proto}://${location.host}/api/monitoring`);

    this.socket.onopen = (_event: Event) => {
      console.debug('Connected');
      this.statusButton.setIcon('sap-icon://connected');
    }

    this.socket.onmessage = (event: MessageEvent) => {
      const monitoring = JSON.parse(event.data);
      this.updateModel(monitoring);
    }

    this.socket.onclose = (_event: CloseEvent) => {
      this.statusButton.setIcon('sap-icon://disconnected');
      console.debug('Disconnected');
    }
  }

  private disconnect() {
    if (this.socket) {
      this.socket.close();
      delete this.socket;
      console.debug('Disconnecting...');
    }
  }
}