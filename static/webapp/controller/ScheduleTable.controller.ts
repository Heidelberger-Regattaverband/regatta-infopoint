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

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ScheduleTableController extends BaseController {

  formatter: Formatter = Formatter;
  private table: Table;
  private readonly scheduleModel: JSONModel = new JSONModel();

  onInit(): void {
    this.table = super.getView()?.byId("scheduleTable") as Table;

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.scheduleModel, "schedule");
    super.getRouter()?.getRoute("schedule")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadScheduleModel(), this);
  }

  onNavBack(): void {
    super.navToStartPage();
  }

  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = (query) ? this.createSearchFilters(query) : [];
    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    this.loadScheduleModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("raceNumber", FilterOperator.Contains, query),
        new Filter("raceShortLabel", FilterOperator.Contains, query),
      ],
      and: false
    })]
  }

  private async loadScheduleModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    return await super.updateJSONModel(this.scheduleModel, `/api/regattas/${regatta.id}/schedule`);
  }
}