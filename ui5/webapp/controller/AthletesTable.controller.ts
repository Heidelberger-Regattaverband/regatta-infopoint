import Button, { Button$PressEvent } from "sap/m/Button";
import ComboBox, { ComboBox$SelectionChangeEvent } from "sap/m/ComboBox";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import { Routes } from "../model/Routes";
import BaseTableController from "./BaseTable.controller";

/**
 * Athlete list supporting text search, club selection, and club-specific route navigation.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class AthletesTable extends BaseTableController {
  private static readonly ATHLETES_MODEL: string = "athletes";
  private static readonly CLUB_FILTER_MODEL: string = "clubFilter";

  private clubFilter?: Filter;
  private textSearchFilters: Filter[] = [];

  /**
   * Registers list models and both athlete routes.
   * Club-route arguments are captured synchronously and applied only after the athlete model has loaded.
   */
  onInit(): void {
    super.init(super.getView()?.byId("athletesTable") as Table);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), AthletesTable.ATHLETES_MODEL);
    super.setViewModel(new JSONModel([]), AthletesTable.CLUB_FILTER_MODEL);
    super
      .getRouter()
      ?.getRoute(Routes.Athletes)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(
          this.loadModel().then(() => {
            this.clearClubFilter();
          }),
          "Failed to load athletes:"
        );
      }, this);

    super
      .getRouter()
      ?.getRoute(Routes.ClubAthletes)
      ?.attachMatched((event: Route$MatchedEvent) => {
        const clubId: number = Number((event.getParameter("arguments") as any)?.clubId);
        super.runAsync(
          this.loadModel().then(() => {
            this.applyClubFilter(clubId);
          }),
          "Failed to load club athletes:"
        );
      }, this);

    super.sortTable("athleteLastNameCol", false, "lastName");
  }

  onNavBack(): void {
    super.navBack(Routes.StartPage);
  }

  /**
   * Updates text search while preserving the current club selection.
   *
   * @param event - Live-change event containing the new search text; blank text clears the text constraint.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    this.textSearchFilters = query ? this.createSearchFilters(query) : [];
    this.applyAllFilters();
  }

  /**
   * Filters by the selected club's short name, combining it with the current text search.
   *
   * @param event - Selection event providing a club short-name key; no selection removes the club constraint.
   */
  onClubSelectionChange(event: ComboBox$SelectionChangeEvent): void {
    const key: string = event.getParameters().selectedItem?.getKey() ?? "";
    this.clubFilter = key ? new Filter("club/shortName", FilterOperator.EQ, key) : undefined;
    this.applyAllFilters();
  }

  /**
   * Opens the athlete sorting dialog, logging fragment-loading failures.
   *
   * @param _event - Sort-button press event; unused because the dialog has no source-specific configuration.
   * @returns A promise resolving after the dialog opens or a loading failure is logged.
   */
  async onSortButtonPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.AthletesSortDialog");
      dialog.open();
    } catch (error) {
      console.error("Failed to load sort dialog:", error);
    }
  }

  /**
   * Reloads athletes and club choices, restoring the refresh button even when loading fails.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   * @returns A promise resolving after loading, feedback, and button restoration, including handled failures.
   */
  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: boolean = await this.loadModel();
      super.showDataUpdatedMessage(succeeded);
    } catch (error) {
      console.error("Failed to refresh athletes:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  /**
   * Opens details for the selected athlete.
   *
   * @param event - Selection event whose list item is bound to an athlete with a backend ID.
   */
  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameters().listItem;
    if (selectedItem) {
      const bindingCtx: Context | undefined | null = selectedItem.getBindingContext("athletes");
      const athlete: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.navToAthleteDetails(athlete.id);
    }
  }

  /**
   * No-op base-table hook; selecting an athlete navigates directly rather than updating shared detail state.
   *
   * @param _item - Selected athlete data; unused because no shared detail state is maintained.
   */
  protected onItemChanged(_item: any): void {
    // nothing to do yet
  }

  /**
   * Resolves a route's club ID to a loaded athlete's club short name and synchronizes the club selector.
   *
   * @param clubId - Backend club ID to locate in the already loaded athlete collection.
   */
  private applyClubFilter(clubId: number): void {
    const athletes: any[] = super.getViewJSONModel(AthletesTable.ATHLETES_MODEL).getData() ?? [];
    const shortName: string | undefined = athletes.find((a: any) => a.club?.id === clubId)?.club?.shortName;
    if (shortName) {
      this.clubFilter = new Filter("club/shortName", FilterOperator.EQ, shortName);
      (super.getView()?.byId("clubComboBox") as ComboBox)?.setSelectedKey(shortName);
      this.applyAllFilters();
    }
  }

  /** Removes a previous club-route filter while retaining text search. */
  private clearClubFilter(): void {
    if (this.clubFilter) {
      this.clubFilter = undefined;
      (super.getView()?.byId("clubComboBox") as ComboBox)?.setSelectedKey("");
      this.applyAllFilters();
    }
  }

  /** Applies text-search alternatives and the optional club constraint together through the base table binding. */
  private applyAllFilters(): void {
    const allFilters: Filter[] = [...this.textSearchFilters];
    if (this.clubFilter) {
      allFilters.push(this.clubFilter);
    }
    super.setSearchFilters(allFilters);
    super.applyFilters();
  }

  /**
   * Builds an OR search over athlete and club names, city, abbreviation, and exact birth year.
   *
   * @param query - Trimmed search text applied to each search alternative.
   * @returns One compound OR filter combining all supported athlete and club search fields.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("lastName", FilterOperator.Contains, query),
          new Filter("firstName", FilterOperator.Contains, query),
          new Filter("year", FilterOperator.EQ, query),
          new Filter("club/shortName", FilterOperator.Contains, query),
          new Filter("club/longName", FilterOperator.Contains, query),
          new Filter("club/abbreviation", FilterOperator.Contains, query),
          new Filter("club/city", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Builds alphabetically sorted club choices, deduplicated by short name, from the loaded athletes.
   *
   * @param athletesModel - Model containing the athlete array from which club choices are derived.
   */
  private populateClubFilterModel(athletesModel: JSONModel): void {
    const data: any = athletesModel.getData();
    const athletes: any[] = Array.isArray(data) ? data : [];
    const seen = new Set<string>();
    const items: { key: string; text: string }[] = [];

    athletes
      .map((a: any) => a.club)
      .filter((club: any) => club?.shortName)
      .sort((a: any, b: any) => (a.shortName as string).localeCompare(b.shortName as string))
      .forEach((club: any) => {
        if (!seen.has(club.shortName as string)) {
          seen.add(club.shortName as string);
          items.push({ key: club.shortName as string, text: club.shortName as string });
        }
      });

    super.getViewJSONModel(AthletesTable.CLUB_FILTER_MODEL).setData(items);
  }

  /**
   * Loads athletes for the active regatta and rebuilds club choices only after a successful request.
   *
   * @returns Whether the athlete request succeeded; active-regatta loading failures still reject.
   */
  private async loadModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url = `/api/regattas/${regatta.id}/athletes`;
    const athletesModel: JSONModel = super.getViewJSONModel(AthletesTable.ATHLETES_MODEL);
    const succeeded: boolean = await super.updateJSONModel(athletesModel, url);

    if (succeeded) {
      this.populateClubFilterModel(athletesModel);
    }

    return succeeded;
  }
}
