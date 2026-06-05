import Log from "sap/base/Log";
import Column from "sap/m/Column";
import ListItemBase from "sap/m/ListItemBase";
import Table from "sap/m/Table";
import Text from "sap/m/Text";
import Toolbar from "sap/m/Toolbar";
import ViewSettingsDialog, { ViewSettingsDialog$ConfirmEvent, ViewSettingsDialog$ConfirmEventParameters } from "sap/m/ViewSettingsDialog";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import CustomData from "sap/ui/core/CustomData";
import EventBus from "sap/ui/core/EventBus";
import Fragment from "sap/ui/core/Fragment";
import { SortOrder } from "sap/ui/core/library";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Sorter from "sap/ui/model/Sorter";
import BaseController from "./Base.controller";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default abstract class BaseTableController extends BaseController {

  /**
   * Cache of already-loaded {@link ViewSettingsDialog} instances keyed by fragment
   * name. Declared as a `static` class field so all derived controllers share the
   * same cache — a fragment loaded by one table view is reused everywhere else.
   */
  private static readonly viewSettingsDialogs: Map<string, ViewSettingsDialog> = new Map<string, ViewSettingsDialog>();

  /**
   * Memoises in-flight {@link Fragment.load} promises per fragment name so that
   * concurrent callers of {@link getViewSettingsDialog} share the same load and
   * do not each create a duplicate dialog (cf. review issue #9). Declared as a
   * `static` class field for the same sharing reason as {@link viewSettingsDialogs}.
   */
  private static readonly viewSettingsDialogPromises: Map<string, Promise<ViewSettingsDialog>> = new Map<string, Promise<ViewSettingsDialog>>();

  protected table: Table;
  private filters: Filter[] = [];
  private searchFilters: Filter[] = [];
  private bindingModel: string;
  /**
   * Identifier of the event-bus channel this controller subscribed to in {@link init}.
   * Stored so {@link onExit} can unsubscribe symmetrically and avoid memory leaks
   * across view destroy/recreate cycles.
   */
  private channelId?: string;

  init(table: Table, channelId?: string): void {
    this.table = table;

    // return the path of the model that is bound to the items, e.g. races or heats
    this.bindingModel = this.table.getBindingInfo("items").model ?? "";

    if (channelId) {
      this.channelId = channelId;
      const bus: EventBus | undefined = super.getEventBus();
      bus?.subscribe(channelId, "first", this.onFirstItemEvent, this);
      bus?.subscribe(channelId, "previous", this.onPreviousItemEvent, this);
      bus?.subscribe(channelId, "next", this.onNextItemEvent, this);
      bus?.subscribe(channelId, "last", this.onLastItemEvent, this);
    }
  }

  /**
   * Unsubscribes from the event-bus channel registered in {@link init} so that
   * the controller (and the `Table` it references) can be garbage-collected
   * when the view is destroyed.
   */
  onExit(): void {
    Log.debug(`BaseTableController.onExit: unsubscribing from event bus channel ${this.channelId}`);
    if (this.channelId) {
      const bus: EventBus | undefined = super.getEventBus();
      bus?.unsubscribe(this.channelId, "first", this.onFirstItemEvent, this);
      bus?.unsubscribe(this.channelId, "previous", this.onPreviousItemEvent, this);
      bus?.unsubscribe(this.channelId, "next", this.onNextItemEvent, this);
      bus?.unsubscribe(this.channelId, "last", this.onLastItemEvent, this);
      this.channelId = undefined;
    }
    super.onExit();
  }

  private onFirstItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    if (index !== 0) {
      this.setCurrentItem(0);
    }
  }

  private onLastItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    this.growTable(400);
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const lastIndex: number = this.table.getItems().length - 1;
    if (index !== lastIndex) {
      this.setCurrentItem(lastIndex);
    }
  }

  private onPreviousItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const previousIndex: number = index > 1 ? index - 1 : 0;
    if (index !== previousIndex) {
      this.setCurrentItem(previousIndex);
    }
  }

  private onNextItemEvent(channelId: string, eventId: string, parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const items: ListItemBase[] = this.table.getItems();
    const nextIndex: number = index < items.length - 1 ? index + 1 : index;
    if (index !== nextIndex) {
      this.growTable(nextIndex);
      this.setCurrentItem(nextIndex);
    }
  }

  async getViewSettingsDialog(dialogFragmentName: string): Promise<ViewSettingsDialog> {
    // Fast path: dialog already loaded and cached.
    const cached: ViewSettingsDialog | undefined = BaseTableController.viewSettingsDialogs.get(dialogFragmentName);
    if (cached) {
      return cached;
    }

    // Memoise the in-flight load so concurrent callers share the same promise
    // instead of each kicking off a duplicate Fragment.load (cf. review issue #9).
    let pending: Promise<ViewSettingsDialog> | undefined = BaseTableController.viewSettingsDialogPromises.get(dialogFragmentName);
    if (!pending) {
      pending = Fragment.load({ id: this.getView()?.getId(), name: dialogFragmentName, controller: this })
        .then((loaded) => {
          const dialog: ViewSettingsDialog = loaded as ViewSettingsDialog;
          dialog.addStyleClass(super.getContentDensityClass());
          this.getView()?.addDependent(dialog);
          BaseTableController.viewSettingsDialogs.set(dialogFragmentName, dialog);
          return dialog;
        })
        .catch((error) => {
          // Drop the rejected promise so a future call can retry the load.
          BaseTableController.viewSettingsDialogPromises.delete(dialogFragmentName);
          throw error;
        });
      BaseTableController.viewSettingsDialogPromises.set(dialogFragmentName, pending);
    }
    return pending;
  }

  onHandleFilterDialogConfirm(event: ViewSettingsDialog$ConfirmEvent): void {
    this.filters = [];

    event.getParameters().filterItems?.forEach((filterItem: ViewSettingsItem) => {
      const customData: CustomData[] = filterItem.getCustomData();
      if (customData) {
        customData.forEach((data: CustomData) => {
          if (data.getKey() === "filter") {
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
    const value2: string | undefined = split[3];
    return new Filter(path, operator, value1, value2);
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
    const params: ViewSettingsDialog$ConfirmEventParameters = event.getParameters()
    const path: string | undefined = params.sortItem?.getKey();

    const customData: CustomData | undefined = params.sortItem?.getCustomData()?.find((data: CustomData) => data.getKey() === "column");
    const columnName: string | undefined = customData?.getValue();
    if (columnName) {
      this.sortTable(columnName, params.sortDescending ?? false, path);
    }
  }

  sortTable(columnName: string, sortDescending: boolean, path?: string) {
    this.table.getColumns().forEach((col: Column) => {
      if (col.getId().endsWith(columnName)) {
        col.setSortIndicator(sortDescending ? SortOrder.Descending : SortOrder.Ascending);
      } else {
        col.setSortIndicator(SortOrder.None);
      }
    });

    const sorters: Sorter[] = [];
    if (path) {
      sorters.push(new Sorter(path, sortDescending));
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
    item._nav = { isFirst: index === 0, isLast: index === items.length - 1 };

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
