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
import ListBinding from "sap/ui/model/ListBinding";
import { Routes } from "../model/Routes";
import BaseTableController from "./BaseTable.controller";

/**
 * Participating-club list with name/location search, sorting, and links to club details.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubsTableController extends BaseTableController {
  private static readonly CLUBS_MODEL: string = "clubs";

  /** Registers route-triggered club loading and sorts the table by city initially. */
  onInit(): void {
    super.init(super.getView()?.byId("clubsTable") as Table);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), ClubsTableController.CLUBS_MODEL);
    super
      .getRouter()
      ?.getRoute(Routes.Clubs)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadModel(), "Failed to load clubs:");
      }, this);

    super.sortTable("clubCityCol", false, "city");
  }

  onNavBack(): void {
    super.navBack(Routes.StartPage);
  }

  /**
   * Applies or clears text-search constraints on the participating-club list.
   *
   * @param event - Live-change event containing the new search text; blank text clears search filters.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding = this.table.getBinding("items") as ListBinding;
    binding.filter(searchFilters);
  }

  /**
   * Opens the club sorting dialog through the shared asynchronous error handler.
   *
   * @param _event - Sort-button press event; unused because the dialog has no source-specific configuration.
   */
  onSortButtonPress(_event: Button$PressEvent): void {
    super.runAsync(
      super.getViewSettingsDialog("de.regatta_hd.infoportal.view.ClubsTableSortDialog").then((dialog) => dialog.open()),
      "Failed to open club sort dialog:"
    );
  }

  /**
   * Reloads clubs and restores the triggering button even when loading rejects.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   */
  onRefreshButtonPress(event: Button$PressEvent): void {
    const source: Button = event.getSource();
    source.setEnabled(false);
    super.runAsync(
      this.loadModel()
        .then((succeeded: boolean) => {
          super.showDataUpdatedMessage(succeeded);
        })
        .finally(() => source.setEnabled(true)),
      "Failed to refresh clubs:"
    );
  }

  /**
   * Opens details for the selected participating club.
   *
   * @param event - Selection event whose list item is bound to a club with a backend ID.
   */
  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("clubs");
      const club: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.navToClubDetails(club.id);
    }
  }

  /**
   * No-op base-table hook; club details are opened directly without shared list-navigation state.
   *
   * @param _item - Selected club data; unused because no shared detail state is maintained.
   */
  protected onItemChanged(_item: any): void {
    // nothing to do yet
  }

  /**
   * Builds an OR search across club names, abbreviation, and city.
   *
   * @param query - Trimmed search text applied to each club search field.
   * @returns One compound OR filter combining all supported club search fields.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("shortName", FilterOperator.Contains, query),
          new Filter("longName", FilterOperator.Contains, query),
          new Filter("abbreviation", FilterOperator.Contains, query),
          new Filter("city", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Loads participating clubs for the active regatta.
   *
   * @returns Whether the club request succeeded; active-regatta loading failures still reject.
   */
  private async loadModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url: string = `/api/regattas/${regatta.id}/clubs`;
    const clubsModel: JSONModel = super.getViewJSONModel(ClubsTableController.CLUBS_MODEL);
    return await super.updateJSONModel(clubsModel, url);
  }
}
