import MessageToast from "sap/m/MessageToast";
import BaseController from "./Base.controller";
import MyComponent from "de/regatta_hd/Component";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import Control from "sap/ui/core/Control";
import { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class Statistics extends BaseController {
  private dataLoader: JSONModel;
  private statisticsModel: JSONModel;
  private racesList?: Control;
  private heatsList?: Control;
  private registrationsList?: Control;
  private athletesList?: Control;

  onInit(): void {
    super.getView()?.addStyleClass((this.getOwnerComponent() as MyComponent).getContentDensityClass());

    super.getRouter()?.getRoute("statistics")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);

    this.statisticsModel = new JSONModel();
    super.setViewModel(this.statisticsModel, "statistics");

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

    // load statistic data from backend
    if (await super.updateJSONModel(this.dataLoader, `/api/regattas/${super.getRegattaId()}/statistics`)) {
      MessageToast.show(super.i18n("msg.dataUpdated"));
    }
    const statistics: any = this.dataLoader.getData();

    // transform statistic data into human readable format
    const registrations = [];
    if (statistics?.registrations) {
      const seats = statistics.registrations.seats + statistics.registrations.seatsCox;
      registrations.push({ name: this.i18n("common.overall"), value: statistics.registrations.all });
      registrations.push({ name: this.i18n("statistics.registrations.cancelled"), value: statistics.registrations.cancelled });
      registrations.push({ name: this.i18n("statistics.reportingClubs"), value: statistics.registrations.registeringClubs });
      registrations.push({ name: this.i18n("statistics.participatingClubs"), value: statistics.registrations.clubs });
      registrations.push({ name: this.i18n("common.athletes"), value: statistics.registrations.athletes });
      registrations.push({ name: this.i18n("common.seats"), value: seats });
    }
    const races = [];
    if (statistics?.races) {
      races.push({ name: this.i18n("common.overall"), value: statistics.races.all });
      races.push({ name: this.i18n("common.cancelled"), value: statistics.races.cancelled });
    }
    const heats = [];
    if (statistics?.heats) {
      heats.push({ name: this.i18n("common.overall"), value: statistics.heats.all });
      heats.push({ name: this.i18n("heat.state.official"), value: statistics.heats.official });
      heats.push({ name: this.i18n("heat.state.finished"), value: statistics.heats.finished });
      heats.push({ name: this.i18n("heat.state.started"), value: statistics.heats.started });
      heats.push({ name: this.i18n("common.seeded"), value: statistics.heats.seeded });
      heats.push({ name: this.i18n("common.scheduled"), value: statistics.heats.scheduled });
      heats.push({ name: this.i18n("common.cancelled"), value: statistics.heats.cancelled });
    }

    const athletes = [];
    if (statistics?.athletes) {
      if (statistics.athletes.oldestWoman) {
        athletes.push({ name: this.i18n("statistics.athletes.oldestWoman"), value: Formatter.athleteLabel(statistics.athletes.oldestWoman) });
      }
      if (statistics.athletes.oldestMan) {
        athletes.push({ name: this.i18n("statistics.athletes.oldestMan"), value: Formatter.athleteLabel(statistics.athletes.oldestMan) });
      }
    }

    // update statistics model
    this.statisticsModel.setProperty("/registrations", registrations);
    this.statisticsModel.setProperty("/races", races);
    this.statisticsModel.setProperty("/heats", heats);
    this.statisticsModel.setProperty("/athletes", athletes);

    this.setBusy(false);
  }

  private setBusy(busy: boolean): void {
    this.registrationsList?.setBusy(busy);
    this.racesList?.setBusy(busy);
    this.heatsList?.setBusy(busy);
    this.athletesList?.setBusy(busy);
  }

}