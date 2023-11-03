import Table from "sap/m/Table";
import BaseController from "./Base.controller";
import Model from "sap/ui/model/Model";
import Filter from "sap/ui/model/Filter";
import ViewSettingsDialog from "sap/m/ViewSettingsDialog";
import Fragment from "sap/ui/core/Fragment";
import Text from "sap/m/Text";
import ListBinding from "sap/ui/model/ListBinding";
import ListItemBase from "sap/m/ListItemBase";
import Event from "sap/ui/base/Event";
import MyComponent from "de/regatta_hd/Component";
import UI5Element from "sap/ui/core/Element";
import Control from "sap/ui/core/Control";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class BaseTable extends BaseController {

  private table: Table;
  private filters: Filter[];
  private searchFilters: Filter[];
  private bindingModel: string;
  private viewSettingsDialogs: Map<string, Promise<ViewSettingsDialog>>;

  public init(table: Table, channelId: string): void {
    // Keeps reference to any of the created sap.m.ViewSettingsDialog-s in this sample
    this.viewSettingsDialogs = new Map<string, Promise<ViewSettingsDialog>>();

    this.table = table;
    this.filters = [];
    this.searchFilters = [];

    // return the path of the model that is bound to the items, e.g. races or heats
    this.bindingModel = this.table.getBindingInfo("items").model || "";

    super.getEventBus()?.subscribe(channelId, "first", this.onFirstItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "previous", this.onPreviousItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "next", this.onNextItemEvent, this);
    super.getEventBus()?.subscribe(channelId, "last", this.onLastItemEvent, this);
  }

  private onFirstItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: int = this.table.indexOfItem(this.table.getSelectedItem());
    if (index != 0) {
      this.setCurrentItem(0);
    }
  }

  private onLastItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    this.growTable(400);
    const index: int = this.table.indexOfItem(this.table.getSelectedItem());
    const lastIndex: int = this.table.getItems().length - 1;
    if (index != lastIndex) {
      this.setCurrentItem(lastIndex);
    }
  }

  private onPreviousItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: int = this.table.indexOfItem(this.table.getSelectedItem());
    const previousIndex: int = index > 1 ? index - 1 : 0;
    if (index != previousIndex) {
      this.setCurrentItem(previousIndex);
    }
  }

  private onNextItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: int = this.table.indexOfItem(this.table.getSelectedItem());
    const items: ListItemBase[] = this.table.getItems();
    const nextIndex: int = index < items.length - 1 ? index + 1 : index;
    if (index != nextIndex) {
      this.growTable(nextIndex);
      this.setCurrentItem(nextIndex);
    }
  }

  getViewSettingsDialog(dialogFragmentName: string): Promise<ViewSettingsDialog> {
    let dialogPromise: Promise<ViewSettingsDialog> = this.viewSettingsDialogs.get(dialogFragmentName)!;
    const that = this;

    if (!dialogPromise) {
      dialogPromise = Fragment.load({
        id: this.getView()?.getId(), name: dialogFragmentName, controller: this
      }).then(function (dialog: Control | Control[]) {
        if (dialog instanceof Control) {
          dialog.addStyleClass((that.getOwnerComponent() as MyComponent).getContentDensityClass());
          that.getView()?.addDependent(dialog);
          return dialog;
        }
      }.bind(that));
      this.viewSettingsDialogs.set(dialogFragmentName, dialogPromise);
    }
    return dialogPromise;
  }

  onHandleFilterDialogConfirm(oEvent: Event): void {
    const mParams = oEvent.getParameters();
    this.filters = [];
    const that = this;

    mParams.filterItems.forEach(function (oItem) {
      const aCustomData = oItem.getCustomData();
      if (aCustomData) {
        aCustomData.forEach(function (oData) {
          if (oData.getKey() == "filter") {
            const oFilter = that.createFilter(oData.getValue());
            that.filters.push(oFilter);
          }
        }.bind(that));
      }
      const oFilter = that.createFilter(oItem.getKey());
      that.filters.push(oFilter);
    }.bind(this));

    // apply filters
    this.applyFilters();

    this.updateFilterBar(mParams.filterString);
  }

  private updateFilterBar(sText: string): void {
    // update filter bar
    const infoToolbar = this.table.getInfoToolbar();
    if (infoToolbar?.getContent()[0]) {
      infoToolbar.setVisible(this.filters.length > 0);
      (infoToolbar.getContent()[0] as Text).setText(sText);
    }
  }

  private createFilter(value: string): Filter {
    const split: string[] = value.split("___");
    const path: string = split[0];
    const operator: string = split[1];
    const sValue1 = split[2] === 'true' || (split[2] === 'false' ? false : split[2]);
    // sValue2 = aSplit[3],
    const filter: Filter = new Filter(path, operator, sValue1);
    return filter;
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

  private setCurrentItem(index: int) {
    const items: ListItemBase[] = this.table.getItems();
    this.table.setSelectedItem(items[index]);

    // gets the selected item in a generic way
    const item = this.table.getSelectedItem().getBindingContext(this.bindingModel).getObject();

    // store navigation meta information in selected item
    item._nav = { isFirst: index == 0, isLast: index == items.length - 1 };

    this.onItemChanged(item);
  }

  onItemChanged(item: any): void {
  }

  private growTable(index: int): void {
    const actual: int = this.table.getGrowingInfo()?.actual || 0;
    if (index >= actual - 5) {
      this.table.setGrowingThreshold(index + 5);
      const allFilters: Filter[] = this.filters.concat(this.searchFilters);
      (this.table.getBinding("items") as ListBinding).filter(allFilters);
    }
  }
};
