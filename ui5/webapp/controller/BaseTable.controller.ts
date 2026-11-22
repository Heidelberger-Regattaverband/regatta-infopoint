import Log from "sap/base/Log";
import Column from "sap/m/Column";
import ListItemBase from "sap/m/ListItemBase";
import Table from "sap/m/Table";
import Text from "sap/m/Text";
import Toolbar from "sap/m/Toolbar";
import ViewSettingsDialog, {
  ViewSettingsDialog$ConfirmEvent,
  ViewSettingsDialog$ConfirmEventParameters
} from "sap/m/ViewSettingsDialog";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import CustomData from "sap/ui/core/CustomData";
import EventBus from "sap/ui/core/EventBus";
import Fragment from "sap/ui/core/Fragment";
import { SortOrder } from "sap/ui/core/library";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import ListBinding from "sap/ui/model/ListBinding";
import Sorter from "sap/ui/model/Sorter";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";

/**
 * Reusable table filtering, sorting, dialog loading, and list-to-detail navigation.
 * Subclasses initialize the table and implement {@link onItemChanged} for their selected data.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default abstract class BaseTableController extends BaseController {
  /**
   * Memoises in-flight {@link Fragment.load} promises per fragment name so that
   * concurrent callers of {@link getViewSettingsDialog} share the same load and
   * do not each create a duplicate dialog. Cached per controller because dialogs belong to its view.
   */
  private readonly viewSettingsDialogPromises: Map<string, Promise<ViewSettingsDialog>> = new Map<
    string,
    Promise<ViewSettingsDialog>
  >();

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
  /**
   * Name of the component-level navigation `JSONModel` (`raceNav` / `heatNav`)
   * that {@link setCurrentItem} updates with `{ isFirst, isLast }`. Keeping the
   * navigation state in a dedicated model — rather than mutating the bound data
   * objects with a `_nav` property — avoids leaking UI metadata into the
   * backend payload.
   */
  private navModelName?: string;

  /**
   * Configures the table binding and optional event-bus navigation.
   *
   * @param table - Table whose items are bound to the list model.
   * @param channelId - Optional channel for first, previous, next, and last selection commands.
   * @param navModelName - Optional component model receiving detail-navigation state.
   */
  protected init(table: Table, channelId?: string, navModelName?: string): void {
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

    this.navModelName = navModelName;
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

  /**
   * Selects the first rendered table row in response to a navigation command.
   *
   * @param _channelId - Publishing event-bus channel; unused because the table is already configured.
   * @param _eventId - Navigation event name; unused by this dedicated handler.
   * @param _parametersMap - Event payload; unused because selection needs no additional arguments.
   */
  private onFirstItemEvent(_channelId: string, _eventId: string, _parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    if (index !== 0) {
      this.setCurrentItem(0);
    }
  }

  /**
   * Expands the growing threshold and selects the last currently rendered table row.
   *
   * @param _channelId - Publishing event-bus channel; unused because the table is already configured.
   * @param _eventId - Navigation event name; unused by this dedicated handler.
   * @param _parametersMap - Event payload; unused because selection needs no additional arguments.
   */
  private onLastItemEvent(_channelId: string, _eventId: string, _parametersMap: any): void {
    this.growTable(400);
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const lastIndex: number = this.table.getItems().length - 1;
    if (index !== lastIndex) {
      this.setCurrentItem(lastIndex);
    }
  }

  /**
   * Selects the preceding table row without moving before the first row.
   *
   * @param _channelId - Publishing event-bus channel; unused because the table is already configured.
   * @param _eventId - Navigation event name; unused by this dedicated handler.
   * @param _parametersMap - Event payload; unused because selection needs no additional arguments.
   */
  private onPreviousItemEvent(_channelId: string, _eventId: string, _parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const previousIndex: number = index > 1 ? index - 1 : 0;
    if (index !== previousIndex) {
      this.setCurrentItem(previousIndex);
    }
  }

  /**
   * Selects the next table row and expands the growing threshold near the loaded boundary.
   *
   * @param _channelId - Publishing event-bus channel; unused because the table is already configured.
   * @param _eventId - Navigation event name; unused by this dedicated handler.
   * @param _parametersMap - Event payload; unused because selection needs no additional arguments.
   */
  private onNextItemEvent(_channelId: string, _eventId: string, _parametersMap: any): void {
    const index: number = this.table.indexOfItem(this.table.getSelectedItem());
    const items: ListItemBase[] = this.table.getItems();
    const nextIndex: number = index < items.length - 1 ? index + 1 : index;
    if (index !== nextIndex) {
      this.growTable(nextIndex);
      this.setCurrentItem(nextIndex);
    }
  }

  /**
   * Loads a settings dialog once per view and attaches it as a dependent.
   * Concurrent calls share the same promise; failed loads are discarded so later calls can retry.
   *
   * @param dialogFragmentName - Fully qualified fragment module name.
   * @returns The cached or newly loaded dialog.
   * @throws The fragment-loading error when loading fails.
   */
  protected async getViewSettingsDialog(dialogFragmentName: string): Promise<ViewSettingsDialog> {
    // Memoise the in-flight load so concurrent callers share the same promise
    // instead of each kicking off a duplicate Fragment.load.
    let pending: Promise<ViewSettingsDialog> | undefined = this.viewSettingsDialogPromises.get(dialogFragmentName);
    if (!pending) {
      pending = Fragment.load({ id: this.getView()?.getId(), name: dialogFragmentName, controller: this })
        .then((loaded) => {
          const dialog: ViewSettingsDialog = loaded as ViewSettingsDialog;
          dialog.addStyleClass(super.getContentDensityClass());
          this.getView()?.addDependent(dialog);
          return dialog;
        })
        .catch((error: any) => {
          // Drop the rejected promise so a future call can retry the load.
          this.viewSettingsDialogPromises.delete(dialogFragmentName);
          throw error;
        });
      this.viewSettingsDialogPromises.set(dialogFragmentName, pending);
    }
    return pending;
  }

  /**
   * Applies encoded dialog filters together with search filters and updates the visible filter summary.
   *
   * @param event - Confirmation event containing selected filter items and their display summary.
   */
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

  /**
   * Updates the filter-summary toolbar and hides it when no dialog filters are selected.
   *
   * @param text - Human-readable filter summary; an empty string clears the summary text.
   */
  private updateFilterBar(text: string): void {
    // update filter bar
    const infoToolbar: Toolbar = this.table.getInfoToolbar();
    if (infoToolbar?.getContent()[0]) {
      infoToolbar.setVisible(this.filters.length > 0);
      (infoToolbar.getContent()[0] as Text).setText(text);
    }
  }

  /**
   * Decodes a dialog key in `path___operator___value1[___value2]` format.
   * The strings `true` and `false` are converted to booleans; other values remain strings.
   *
   * @param value - Encoded binding path, operator, and one or two comparison values from a dialog item.
   * @returns A UI5 filter built from the decoded components.
   */
  private createFilter(value: string): Filter {
    const split: string[] = value.split("___");
    const path: string = split[0];
    const operator: FilterOperator = split[1] as FilterOperator;
    const value1: string | boolean = split[2] === "true" || (split[2] === "false" ? false : split[2]);
    const value2: string | undefined = split[3];
    return new Filter(path, operator, value1, value2);
  }

  /**
   * Replaces search filters without applying them; call {@link applyFilters} to update the binding.
   *
   * @param searchFilters - Replacement search constraints; an empty array removes text-search restrictions.
   */
  protected setSearchFilters(searchFilters: Filter[]): void {
    this.searchFilters = searchFilters;
  }

  /** Clears dialog filters and their summary, leaving search filters unchanged until the next application. */
  protected clearFilters(): void {
    this.filters = [];
    this.updateFilterBar("");
  }

  /** Applies the combined dialog and search filters to the table's items binding. */
  protected applyFilters(): void {
    // combine search and filters from dialog
    const allFilters: Filter[] = this.filters.concat(this.searchFilters);
    (this.table.getBinding("items") as ListBinding).filter(allFilters);
  }

  /**
   * Reads the selected column's custom data and applies its sort direction and binding path.
   *
   * @param event - Confirmation event containing the chosen sort item and descending flag.
   */
  onSortDialogConfirm(event: ViewSettingsDialog$ConfirmEvent): void {
    const params: ViewSettingsDialog$ConfirmEventParameters = event.getParameters();
    const path: string | undefined = params.sortItem?.getKey();

    const customData: CustomData | undefined = params.sortItem
      ?.getCustomData()
      ?.find((data: CustomData) => data.getKey() === "column");
    const columnName: string | undefined = customData?.getValue();
    if (columnName) {
      this.sortTable(columnName, params.sortDescending ?? false, path);
    }
  }

  /**
   * Updates column sort indicators and sorts the items binding.
   *
   * @param columnName - Suffix identifying the selected column's control ID.
   * @param sortDescending - Whether to sort in descending order.
   * @param path - Binding property to sort; omitting it clears binding sorters.
   */
  protected sortTable(columnName: string, sortDescending: boolean, path?: string) {
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

  /**
   * Selects a rendered row, resets detail-navigation state, and passes its bound data to the subclass.
   *
   * @param index - Zero-based index of an existing rendered item in the table.
   */
  private setCurrentItem(index: number): void {
    const items: ListItemBase[] = this.table.getItems();
    this.table.setSelectedItem(items[index]);

    // gets the selected item in a generic way
    const item: any = this.table.getSelectedItem()?.getBindingContext(this.bindingModel)?.getObject();

    // Store navigation meta-information in the dedicated nav model,
    // not on the data object. The disabled/back fields are reset because navigating
    // through the table always re-enables the nav buttons in the detail view.
    this.updateNavModel({ isFirst: index === 0, isLast: index === items.length - 1, disabled: false, back: undefined });

    this.onItemChanged(item);
  }

  /**
   * Writes the navigation state to the configured nav `JSONModel` (e.g. `raceNav`
   * or `heatNav`). No-op when no nav model name was passed to {@link init}.
   *
   * @param state - Complete replacement navigation state for the configured component model.
   */
  protected updateNavModel(state: NavigationData): void {
    if (!this.navModelName) {
      return;
    }
    super.getComponentJSONModel(this.navModelName).setData(state);
  }

  /**
   * Subclass hook invoked after list navigation selects a row.
   *
   * @param item - Data object from the selected row's binding context.
   */
  protected abstract onItemChanged(item: any): void;

  /**
   * Raises the growing threshold when selection approaches the end of the currently loaded rows.
   *
   * @param index - Zero-based row index that navigation is preparing to reach.
   */
  private growTable(index: number): void {
    const actual: number = this.table.getGrowingInfo()?.actual || 0;
    if (index >= actual - 5) {
      this.table.setGrowingThreshold(index + 5);
      const allFilters: Filter[] = this.filters.concat(this.searchFilters);
      (this.table.getBinding("items") as ListBinding).filter(allFilters);
    }
  }
}
