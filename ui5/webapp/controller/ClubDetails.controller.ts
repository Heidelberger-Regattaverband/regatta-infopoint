import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import { Route$PatternMatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import ListBinding from "sap/ui/model/ListBinding";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import { NavigationData } from "../model/types";
import BaseController from "./Base.controller";
import RacesTableController from "./RacesTable.controller";

/**
 * Displays a club and its entries, with links to club athletes, location, and race details.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class ClubDetailsController extends BaseController {
  private static readonly CLUB_MODEL: string = "club";
  private static readonly ENTRIES_MODEL: string = "entries";

  readonly formatter: Formatter = Formatter;
  private table?: Table;
  private clubId?: number;

  /** Registers club and entry models and loads them when the club route matches. */
  onInit(): void {
    super.getView()?.addStyleClass(super.getContentDensityClass());

    this.table = super.getView()?.byId("clubEntriesTable") as Table;

    super.setViewModel(new JSONModel(), ClubDetailsController.ENTRIES_MODEL);
    super.setViewModel(new JSONModel(), ClubDetailsController.CLUB_MODEL);

    super
      .getRouter()
      ?.getRoute(Routes.ClubDetails)
      ?.attachPatternMatched((event: Route$PatternMatchedEvent) => {
        super.runAsync(this.onPatternMatched(event), "Failed to load club details:");
      }, this);
  }

  onNavBack(): void {
    super.navBack(Routes.Clubs);
    delete this.clubId;
  }

  onCityPress(): void {
    super.getRouter()?.navTo(Routes.MapClub, { clubId: this.clubId });
  }

  onAthletesPress(): void {
    super.navToClubAthletes(this.clubId!);
  }

  /**
   * Opens the selected entry's race with list-navigation controls hidden and club details as the back target.
   *
   * @param oEvent - Selection event whose list item is bound to a club race entry.
   */
  onSelectionChange(oEvent: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = oEvent.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext(
        ClubDetailsController.ENTRIES_MODEL
      );
      const entry: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());
      super.getComponentJSONModel(RacesTableController.RACE_MODEL).setData(entry.race);

      const navData: NavigationData = { isFirst: false, isLast: false, disabled: true, back: Routes.ClubDetails };
      super.getComponentJSONModel(RacesTableController.RACE_NAV_MODEL).setData(navData);

      super.navToRaceDetails(entry.race.id);
    }
  }

  /**
   * Refreshes both models and restores the triggering button whether loading succeeds or fails.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   * @returns A promise resolving after loading, feedback, and button restoration, including handled failures.
   */
  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: [boolean, boolean] = await this.loadData();
      super.showDataUpdatedMessage(succeeded[0] && succeeded[1]);
    } catch (error) {
      console.error("Failed to refresh club details:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  /**
   * Applies or clears text-search constraints on the club's entries table.
   *
   * @param event - Live-change event containing the new search text; blank text clears search filters.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    const searchFilters: Filter[] = query ? this.createSearchFilters(query) : [];

    const binding: ListBinding | undefined = this.table?.getBinding("items") as ListBinding;
    binding?.filter(searchFilters);
  }

  /**
   * Matches any crew member's name case-insensitively or the entry's race number.
   *
   * @param query - Trimmed search text applied to crew names and race numbers.
   * @returns One compound OR filter combining crew-name and race-number alternatives.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter({
            path: "crew/",
            test: function (crews: any[]) {
              for (const crew of crews) {
                const found =
                  crew.athlete.firstName.toLowerCase().includes(query.toLowerCase()) ||
                  crew.athlete.lastName.toLowerCase().includes(query.toLowerCase());
                if (found) {
                  return true;
                }
              }
              return false;
            }
          }),
          new Filter("race/number", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Captures the club ID before awaiting the initial detail load.
   *
   * @param event - Route match containing `clubId` in its arguments.
   * @returns A promise resolving after both detail requests settle; regatta-lookup failures reject.
   */
  private async onPatternMatched(event: Route$PatternMatchedEvent): Promise<void> {
    this.clubId = (event.getParameter("arguments") as any).clubId;
    await this.loadData();
  }

  /**
   * Refreshes club details and entries in parallel for the active regatta.
   *
   * @returns Update results in `[entries, club]` order.
   */
  private async loadData(): Promise<[boolean, boolean]> {
    const regatta: any = await super.getActiveRegatta();

    const clubUrl: string = `/api/regattas/${regatta.id}/clubs/${this.clubId}`;
    const entriesUrl: string = `/api/regattas/${regatta.id}/clubs/${this.clubId}/entries`;

    const clubModel: JSONModel = super.getViewJSONModel(ClubDetailsController.CLUB_MODEL);
    const entriesModel: JSONModel = super.getViewJSONModel(ClubDetailsController.ENTRIES_MODEL);

    return await Promise.all([
      super.updateJSONModel(entriesModel, entriesUrl),
      super.updateJSONModel(clubModel, clubUrl)
    ]);
  }
}
