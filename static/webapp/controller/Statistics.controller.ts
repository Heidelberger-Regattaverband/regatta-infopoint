import { Button$PressEvent } from "sap/m/Button";
import GroupHeaderListItem from "sap/m/GroupHeaderListItem";
import Table from "sap/m/Table";
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
  private statisticsTable?: Table;

  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    super.getRouter()?.getRoute("statistics")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadStatistics(), this);

    super.setViewModel(this.statisticsModel, "statistics");

    this.statisticsTable = this.getView()?.byId("statisticsTable") as Table;
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

    const all = [];

    if (statistics?.heats) {
      all.push({ name: this.i18n("common.overall"), value: statistics.heats.all, group: "1" },
        { name: this.i18n("heat.state.official"), value: statistics.heats.official, group: "1" },
        { name: this.i18n("heat.state.finished"), value: statistics.heats.finished, group: "1" },
        { name: this.i18n("heat.state.started"), value: statistics.heats.started, group: "1" },
        { name: this.i18n("common.seeded"), value: statistics.heats.seeded, group: "1" },
        { name: this.i18n("common.scheduled"), value: statistics.heats.scheduled, group: "1" },
        { name: this.i18n("common.cancelled"), value: statistics.heats.cancelled, group: "1" });
    }

    if (statistics?.entries) {
      const seats = statistics.entries.seats + statistics.entries.seatsCox;
      all.push({ name: this.i18n("common.overall"), value: statistics.entries.all, group: "2" },
        { name: this.i18n("statistics.entries.cancelled"), value: statistics.entries.cancelled, group: "2" },
        { name: this.i18n("statistics.reportingClubs"), value: statistics.entries.registeringClubs, group: "2" },
        { name: this.i18n("statistics.participatingClubs"), value: statistics.entries.clubs, group: "2" },
        { name: this.i18n("statistics.athletes.overall"), value: statistics.entries.athletes, group: "2" },
        { name: this.i18n("statistics.athletes.female"), value: statistics.entries.athletesFemale, group: "2" },
        { name: this.i18n("statistics.athletes.male"), value: statistics.entries.athletesMale, group: "2" },
        { name: this.i18n("common.seats"), value: seats, group: "2" });
    }

    if (statistics?.races) {
      all.push({ name: this.i18n("common.overall"), value: statistics.races.all, group: "3" },
        { name: this.i18n("common.cancelled"), value: statistics.races.cancelled, group: "3" });
    }

    if (statistics?.medals) {
      all.push({ name: this.i18n("statistics.medals.rowers"), value: statistics.medals.rowers, group: "4" },
        { name: this.i18n("statistics.medals.coxes"), value: statistics.medals.coxes, group: "4" });
    }

    if (statistics?.athletes) {
      if (statistics.athletes.oldestWoman) {
        all.push({ name: this.i18n("statistics.athletes.oldestWoman"), value: Formatter.athleteLabel(statistics.athletes.oldestWoman), group: "5" });
      }
      if (statistics.athletes.oldestMan) {
        all.push({ name: this.i18n("statistics.athletes.oldestMan"), value: Formatter.athleteLabel(statistics.athletes.oldestMan), group: "5" });
      }
    }
    this.statisticsModel.setData(all);

    return succeeded;
  }

  getGroup(context: any): string {
    return context.getProperty("group");
  }

  getGroupHeader(group: any): any {
    let title: string = "";
    switch (group?.key) {
      case "1":
        title = this.i18n("statistics.heats");
        break;
      case "2":
        title = this.i18n("common.entries");
        break;
      case "3":
        title = this.i18n("statistics.races");
        break;
      case "5":
        title = this.i18n("statistics.athletes");
        break;
      case "4":
        title = this.i18n("statistics.medals");
        break;
    }
    return new GroupHeaderListItem({
      title: title,
    });
  }

  private setBusy(busy: boolean): void {
    this.statisticsTable?.setBusy(busy);
  }
}