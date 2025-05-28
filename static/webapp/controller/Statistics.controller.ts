import { Button$PressEvent } from "sap/m/Button";
import List from "sap/m/List";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class StatisticsController extends BaseController {

  private readonly dataLoader: JSONModel = new JSONModel();
  private readonly statisticsModel: JSONModel = new JSONModel();
  private racesList?: List;
  private heatsList?: List;
  private entriesList?: List;
  private athletesList?: List;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    super.getRouter()?.getRoute("statistics")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);

    super.setViewModel(this.statisticsModel, "statistics");

    this.entriesList = this.getView()?.byId("entriesList") as List;
    this.racesList = this.getView()?.byId("racesList") as List;
    this.heatsList = this.getView()?.byId("heatsList") as List;
    this.athletesList = this.getView()?.byId("athletesList") as List;
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    this.setBusy(true);
    this.loadStatistics().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => {
      this.setBusy(false);
    });
  }

  private async loadStatistics(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    // load statistic data from backend
    const succeeded: boolean = await super.updateJSONModel(this.dataLoader, `/api/regattas/${regatta.id}/statistics`);
    let statistics: any = succeeded ? this.dataLoader.getData() : {};

    // transform statistic data into human readable format
    const entries = [];
    if (statistics?.entries) {
      const seats = statistics.entries.seats + statistics.entries.seatsCox;
      entries.push({ name: this.i18n("common.overall"), value: statistics.entries.all });
      entries.push({ name: this.i18n("statistics.registrations.cancelled"), value: statistics.entries.cancelled });
      entries.push({ name: this.i18n("statistics.reportingClubs"), value: statistics.entries.registeringClubs });
      entries.push({ name: this.i18n("statistics.participatingClubs"), value: statistics.entries.clubs });
      entries.push({ name: this.i18n("statistics.athletes.overall"), value: statistics.entries.athletes });
      entries.push({ name: this.i18n("statistics.athletes.female"), value: statistics.entries.athletesFemale });
      entries.push({ name: this.i18n("statistics.athletes.male"), value: statistics.entries.athletesMale });
      entries.push({ name: this.i18n("common.seats"), value: seats });
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
    this.statisticsModel.setProperty("/entries", entries);
    this.statisticsModel.setProperty("/races", races);
    this.statisticsModel.setProperty("/heats", heats);
    this.statisticsModel.setProperty("/athletes", athletes);

    return succeeded;
  }

  private setBusy(busy: boolean): void {
    this.entriesList?.setBusy(busy);
    this.racesList?.setBusy(busy);
    this.heatsList?.setBusy(busy);
    this.athletesList?.setBusy(busy);
  }
}