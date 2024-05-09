import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import Filter from "sap/ui/model/Filter";
import ViewSettingsDialog, { ViewSettingsDialog$ConfirmEvent, ViewSettingsDialog$ConfirmEventParameters } from "sap/m/ViewSettingsDialog";
import Fragment from "sap/ui/core/Fragment";
import Text from "sap/m/Text";
import ListBinding from "sap/ui/model/ListBinding";
import ListItemBase from "sap/m/ListItemBase";
import FilterOperator from "sap/ui/model/FilterOperator";
import CustomData from "sap/ui/core/CustomData";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import Toolbar from "sap/m/Toolbar";
import Sorter from "sap/ui/model/Sorter";
import Column from "sap/m/Column";
import { SortOrder } from "sap/ui/core/library";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default abstract class BaseTableController extends BaseController {

  protected table: Table;
  private filters: Filter[];
  private searchFilters: Filter[];
  private bindingModel: string;
  private viewSettingsDialogs: Map<string, ViewSettingsDialog>;

  init(table: Table, channelId: string): void {
    // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
    this.viewSettingsDialogs = new Map<string, ViewSettingsDialog>();

    this.table = table;
    this.filters = [];
    this.searchFilters = [];

    // return the path of the model that is bound to the items, e.g. races or heats
    this.bindingModel = this.table.getBindingInfo("items").model ?? "";

    super.getEventBus()?.subscribe(channelId, "first", this.onFirstItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "previous", this.onPreviousItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "next", this.onNextItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "last", this.onLastItemEvent, this);
  }

  private onFirstItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    if (index != 0) {
      this.setCurrentItem(0);
    }
  }

  private onLastItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    this.growTable(400);
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const lastIndex: number = this.table.getItems().length - 1;
    if (index != lastIndex) {
      this.setCurrentItem(lastIndex);
    }
  }

  private onPreviousItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const previousIndex: number = index > 1 ? index - 1 : 0;
    if (index != previousIndex) {
      this.setCurrentItem(previousIndex);
    }
  }

  private onNextItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const items: ListItemBase[] = this.table.getItems();
    const nextIndex: number = index < items.length - 1 ? index + 1 : index;
    if (index != nextIndex) {
      this.growTable(nextIndex);
      this.setCurrentItem(nextIndex);
    }
  }

  async getViewSettingsDialog(dialogFragmentName: string): Promise<ViewSettingsDialog> {
    let dialog: ViewSettingsDialog = this.viewSettingsDialogs.get(dialogFragmentName)!;

    if (!dialog) {
      dialog = await Fragment.load({ id: this.getView()?.getId(), name: dialogFragmentName, controller: this }) as ViewSettingsDialog;
      dialog.addStyleClass(super.getContentDensityClass());
      this.getView()?.addDependent(dialog);
      this.viewSettingsDialogs.set(dialogFragmentName, dialog);
    }
    return dialog;
  }

  onHandleFilterDialogConfirm(event: ViewSettingsDialog$ConfirmEvent): void {
    this.filters = [];

    event.getParameters().filterItems?.forEach((filterItem: ViewSettingsItem) => {
      const customData: CustomData[] = filterItem.getCustomData();
      if (customData) {
        customData.forEach((data: CustomData) => {
          if (data.getKey() == "filter") {
            const filter = this.createFilter(data.getValue());
            this.filters.push(filter);
          }
        });
      }
      const filter: Filter = this.createFilter(filterItem.getKey());
      this.filters.push(filter);
    });

    // apply filters
    this.applyFilters();

    this.updateFilterBar((event.getParameters() as any).filterString);
  }

  private updateFilterBar(text: string): void {
    // update filter bar
    const infoToolbar: Toolbar = this.table.getInfoToolbar();
    if (infoToolbar?.getContent()[0]) {
      infoToolbar.setVisible(this.filters.length > 0);
      (infoToolbar.getContent()[0] as Text).setText(text);
    }
  }

  private createFilter(value: string): Filter {
    const split: string[] = value.split("___");
    const path: string = split[0];
    const operator: FilterOperator = split[1] as FilterOperator;
    const value1: string | boolean = split[2] === 'true' || (split[2] === 'false' ? false : split[2]);
    // sValue2 = aSplit[3],
    return new Filter(path, operator, value1);
  }

  setSearchFilters(searchFilters: Filter[]): void {
    this.searchFilters = searchFilters;
  }

  clearFilters(): void {
    this.filters = [];
    this.updateFilterBar("");
  }

  applyFilters(): void {
    // combine search and filters from dialog
    const allFilters: Filter[] = this.filters.concat(this.searchFilters);
    (this.table.getBinding("items") as ListBinding).filter(allFilters);
  }

  onSortDialogConfirm(event: ViewSettingsDialog$ConfirmEvent): void {
    const sorters: Sorter[] = [];
    const params: ViewSettingsDialog$ConfirmEventParameters = event.getParameters()
    const path: string | undefined = params.sortItem?.getKey();

    const customData: CustomData | undefined = params.sortItem?.getCustomData()?.find((data: CustomData) => data.getKey() === "column");
    const columnName: string = customData?.getValue();
    if (columnName) {
      this.table.getColumns().forEach((col: Column) => {
        if (col.getId().endsWith(columnName)) {
          col.setSortIndicator(params.sortDescending ? SortOrder.Descending : SortOrder.Ascending);
        } else {
          col.setSortIndicator(SortOrder.None);
        }
      })
    }
    if (path) {
      sorters.push(new Sorter(path, params.sortDescending));
    }

    // apply the selected sort and group settings
    (this.table.getBinding("items") as ListBinding).sort(sorters);
  }


  private setCurrentItem(index: number): void {
    const items: ListItemBase[] = this.table.getItems();
    this.table.setSelectedItem(items[index]);

    // gets the selected item in a generic way
    const item: any = this.table.getSelectedItem()?.getBindingContext(this.bindingModel)?.getObject();

    // store navigation meta information in selected item
    item._nav = { isFirst: index == 0, isLast: index == items.length - 1 };

    this.onItemChanged(item);
  }

  abstract onItemChanged(item: any): void;

  private growTable(index: number): void {
    const actual: number = this.table.getGrowingInfo()?.actual || 0;
    if (index >= actual - 5) {
      this.table.setGrowingThreshold(index + 5);
      const allFilters: Filter[] = this.filters.concat(this.searchFilters);
      (this.table.getBinding("items") as ListBinding).filter(allFilters);
    }
  }

}
