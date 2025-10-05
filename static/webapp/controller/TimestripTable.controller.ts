import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
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
export default class TimestripTableController extends BaseTableController {

  static readonly TIMESTAMP_MODEL: string = "timestamp";
  private static readonly TIMESTRIP_MODEL: string = "timestrip";

  readonly formatter: Formatter = Formatter;

  onInit(): void {
    super.init(super.getView()?.byId("timestripTable") as Table, "timestamp" /* eventBus channel */);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), TimestripTableController.TIMESTRIP_MODEL);
    super.getRouter()?.getRoute("timestrip")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadTimestripModel(), this);
  }

  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("timestrip");
      const timestamp: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      const index: number = this.table.indexOfItem(selectedItem);
      const count = this.table.getItems().length;
      // store navigation meta information in selected item
      timestamp._nav = { isFirst: index == 0, isLast: index == count - 1 };

      this.onItemChanged(timestamp);
    }
  }

  onNavBack(): void {
    super.navToStartPage();
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
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
    this.loadTimestripModel().then((succeeded: boolean) => {
      super.showDataUpdatedMessage(succeeded);
    }).finally(() => source.setEnabled(true));
  }

  onItemChanged(item: any): void {
    super.getComponentJSONModel(TimestripTableController.TIMESTAMP_MODEL).setData(item);
    super.getEventBus()?.publish("timestamp", "itemChanged", {});
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

  private async loadTimestripModel(): Promise<boolean> {
    const url: string = `/api/regattas/active/timestrip`;
    const timestripModel: JSONModel = super.getViewJSONModel(TimestripTableController.TIMESTRIP_MODEL);
    return await super.updateJSONModel(timestripModel, url);
  }
}
