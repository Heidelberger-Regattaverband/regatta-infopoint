import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import MessageToast from "sap/m/MessageToast";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseTableController from "./BaseTable.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class TimekeepingController extends BaseTableController {

  private static readonly TIMESTRIP_MODEL: string = "timestrip";
  private static readonly AQUARIUS_HEATS_MODEL: string = "aquariusHeats";

  readonly formatter: Formatter = Formatter;
  private socket?: WebSocket;
  private statusButton?: Button;
  // bind keyListener method to this context to have access to navigation methods
  private readonly keyListener: (event: KeyboardEvent) => void = this.onKeyDown.bind(this);

  onInit(): void {
    super.init(super.getView()?.byId("timestripTable") as Table, "timestamp" /* eventBus channel */);
    this.statusButton = this.byId("timekeepingStatusButton") as Button;
    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.getView()?.addEventDelegate({ onBeforeHide: this.onBeforeHide }, this);
    super.setViewModel(new JSONModel(), TimekeepingController.TIMESTRIP_MODEL);
    super.setViewModel(new JSONModel(), TimekeepingController.AQUARIUS_HEATS_MODEL);
    super.getRouter()?.getRoute("timekeeping")?.attachMatched((_: Route$MatchedEvent) => {
      this.connect();
      this.loadTimestripModel();
    }, this);
  }

  private onBeforeHide(): void {
    this.disconnect();
  }

  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(TimekeepingController.TIMESTRIP_MODEL);
      const timestamp: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      this.onItemChanged(timestamp);
    }
  }

  onNavBack(): void {
    super.navToStartPage();
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
  }

  onStatusButtonPress(): void {
    this.connect();
  }

  onStartButtonPress(): void {
    this.sendCommand({ AddStart: null });
  }

  onFinishButtonPress(): void {
    this.sendCommand({ AddFinish: null });
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    if (query) {
      super.setSearchFilters(this.createSearchFilters(query));
    } else {
      super.setSearchFilters([]);
    }
    super.applyFilters();
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadTimestripModel();
    source.setEnabled(true);
  }

  onItemChanged(item: any): void {
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("heat_nr", FilterOperator.Contains, query),
        new Filter("bib", FilterOperator.Contains, query),
        new Filter("split", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private loadTimestripModel() {
    this.sendCommand({ GetTimestrip: null });
  }

  private updateModel(timekeeping: any) {
  }

  private connect() {
    this.disconnect();

    const location: Location = globalThis.location;
    const proto = location.protocol.startsWith('https') ? 'wss' : 'ws';

    console.debug('Connecting Timekeeping WebSocket ...');
    this.socket = new WebSocket(`${proto}://${location.host}/api/timekeeping`);

    this.socket.onopen = (_event: Event) => {
      this.statusButton?.setIcon('sap-icon://connected');
      console.debug('Timekeeping WebSocket Connected');
    }
    this.socket.onclose = (_event: CloseEvent) => {
      this.statusButton?.setIcon('sap-icon://disconnected');
      console.debug('Timekeeping WebSocket Disconnected');
    }
    this.socket.onmessage = (msgEvent: MessageEvent) => {
      const data: any = JSON.parse(msgEvent.data);
      if (data.error) {
        console.error('Timekeeping WebSocket error:', data.error);
        super.showErrorMessageToast(data.error);
      } else
        if (data.AquariusHeats) {
          super.getViewJSONModel(TimekeepingController.AQUARIUS_HEATS_MODEL).setData(data.AquariusHeats.heats);
        } else if (data.Timestamp) {
          super.showInfoMessageToast("Timestamp added successfully");
        } else if (data.Timestrip) {
          super.getViewJSONModel(TimekeepingController.TIMESTRIP_MODEL).setData(data.Timestrip.time_stamps);
          super.showInfoMessageToast("Timestrip retrieved successfully");
        } else {
          console.warn(`Received unknown Timekeeping WebSocket event type: ${JSON.stringify(data)}`);
        }
    }
  }

  private disconnect() {
    if (this.socket) {
      console.debug('Disconnecting Timekeeping WebSocket ...');
      this.socket.close();
      delete this.socket;
    }
  }

  private sendCommand(command: any) {
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify(command));
    }
  }

  private onKeyDown(event: KeyboardEvent): void {
    switch (event.key) {
      case "Space":
        event.preventDefault();
        break;
      case "ArrowLeft":
        break;
      case "ArrowRight":
        break;
      case "ArrowUp":
      case "Home":
        break;
      case "ArrowDown":
      case "End":
        break;
    }
  }
}
