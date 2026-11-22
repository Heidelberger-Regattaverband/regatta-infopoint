import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Button, { Button$PressEvent } from "sap/m/Button";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";

/**
 * Active-regatta schedule table with race-label search and manual refresh.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ScheduleTableController extends BaseController {
  formatter: Formatter = Formatter;
  private table: Table;
  private readonly scheduleModel: JSONModel = new JSONModel();

  /** Registers the schedule model and refreshes it when the schedule route matches. */
  onInit(): void {
    this.table = super.getView()?.byId("scheduleTable") as Table;

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.scheduleModel, "schedule");
    super
      .getRouter()
      ?.getRoute(Routes.Schedule)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadScheduleModel(), "Failed to load schedule:");
      }, this);
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  /**
   * Applies or clears text-search constraints on the schedule table.
   *
   * @param event - Live-change event containing new search text; blank text clears search filters.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  /**
   * Reloads the schedule and restores the triggering button even when loading rejects.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   */
  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadScheduleModel()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh schedule:"
    );
  }

  /**
   * Builds an OR search across race number and short label.
   *
   * @param query - Trimmed search text applied to the schedule's race fields.
   * @returns One compound OR filter combining race-number and short-label alternatives.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("raceNumber", FilterOperator.Contains, query),
          new Filter("raceShortLabel", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Requests the schedule for the active regatta.
   *
   * @returns Whether the schedule request succeeded; active-regatta loading failures still reject.
   */
  private async loadScheduleModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    return await super.updateJSONModel(this.scheduleModel, `/api/regattas/${regatta.id}/schedule`);
  }
}
