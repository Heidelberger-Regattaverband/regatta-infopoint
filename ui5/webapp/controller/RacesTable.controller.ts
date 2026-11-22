import Button, { Button$PressEvent } from "sap/m/Button";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import ListItemBase from "sap/m/ListItemBase";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import Table from "sap/m/Table";
import ViewSettingsDialog from "sap/m/ViewSettingsDialog";
import ViewSettingsFilterItem from "sap/m/ViewSettingsFilterItem";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import Context from "sap/ui/model/Context";
import Filter from "sap/ui/model/Filter";
import FilterOperator from "sap/ui/model/FilterOperator";
import JSONModel from "sap/ui/model/json/JSONModel";
import Formatter from "../model/Formatter";
import { Routes } from "../model/Routes";
import { NavigationData } from "../model/types";
import BaseTableController from "./BaseTable.controller";

/**
 * Race list with search, regatta-specific filters, sorting, and selection shared with race details.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class RacesTableController extends BaseTableController {
  private static readonly FILTER_DIALOG: string = "de.regatta_hd.infoportal.view.RacesFilterDialog";
  private static readonly SORT_DIALOG: string = "de.regatta_hd.infoportal.view.RacesSortDialog";
  /** Component model containing the race selected for the detail view. */
  static readonly RACE_MODEL: string = "race";
  /** Component model containing list-navigation state, kept separate from race data. */
  static readonly RACE_NAV_MODEL: string = "raceNav";
  private static readonly RACES_MODEL: string = "races";

  readonly formatter: Formatter = Formatter;

  /** Configures table navigation, route-triggered loading, default sorting, and dynamic filter options. */
  onInit(): void {
    super.init(
      super.getView()?.byId("racesTable") as Table,
      "race" /* eventBus channel */,
      RacesTableController.RACE_NAV_MODEL
    );

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), RacesTableController.RACES_MODEL);
    super
      .getRouter()
      ?.getRoute(Routes.Races)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadRacesModel(), "Failed to load races:");
      }, this);

    super.sortTable("raceNumber", false, "number");

    void this.initFilterDialog();
  }

  /**
   * Stores the selected race and its list position before opening details.
   *
   * @param event - Selection event whose list item is bound to the race to display.
   */
  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("races");
      const race: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      const index: number = this.table.indexOfItem(selectedItem);
      const count: number = this.table.getItems().length;
      const navData: NavigationData = {
        isFirst: index === 0,
        isLast: index === count - 1,
        disabled: false,
        back: undefined
      };
      super.updateNavModel(navData);

      this.onItemChanged(race);
      super.navToRaceDetails(race.id);
    }
  }

  /** Returns to the launchpad and reduces the growing threshold for the next visit. */
  onNavBack(): void {
    super.navToStartPage();
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
  }

  /**
   * Applies trimmed text search together with the active dialog filters.
   *
   * @param event - Live-change event containing new search text; blank text removes only the search constraint.
   */
  onSearchFieldLiveChange(event: SearchField$LiveChangeEvent): void {
    const query: string | undefined = event.getParameters().newValue?.trim();
    if (query) {
      super.setSearchFilters(this.createSearchFilters(query));
    } else {
      super.setSearchFilters([]);
    }
    super.applyFilters();
  }

  /**
   * Opens the race filtering dialog, logging fragment-loading failures.
   *
   * @param _event - Filter-button press event; unused because the dialog is fixed for this controller.
   * @returns A promise resolving after the dialog opens or a loading failure is logged.
   */
  async onFilterButtonPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog(RacesTableController.FILTER_DIALOG);
      dialog.open();
    } catch (error) {
      console.error("Failed to load filter dialog:", error);
    }
  }

  /**
   * Clears dialog filters without discarding the current text search, even if the dialog cannot load.
   *
   * @param _event - Clear-filter press event; unused because all dialog filters are reset.
   * @returns A promise resolving after dialog reset is attempted and the remaining search filters are applied.
   */
  async onClearFilterPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog(RacesTableController.FILTER_DIALOG);
      dialog.clearFilters();
    } catch (error) {
      console.error("Failed to load filter dialog:", error);
    }
    super.clearFilters();
    super.applyFilters();
  }

  /**
   * Opens the race sorting dialog, logging fragment-loading failures.
   *
   * @param _event - Sort-button press event; unused because the dialog is fixed for this controller.
   * @returns A promise resolving after the dialog opens or a loading failure is logged.
   */
  async onSortButtonPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog(RacesTableController.SORT_DIALOG);
      dialog.open();
    } catch (error) {
      console.error("Failed to load sort dialog:", error);
    }
  }

  /**
   * Reloads races and restores the refresh button on both success and failure.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   * @returns A promise resolving after loading, feedback, and button restoration, including handled failures.
   */
  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: boolean = await this.loadRacesModel();
      super.showDataUpdatedMessage(succeeded);
    } catch (error) {
      console.error("Failed to refresh races:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  /**
   * Updates the shared race model and notifies the visible detail view through the `race` event-bus channel.
   *
   * @param item - Selected race data to place in the component's shared race model.
   */
  protected onItemChanged(item: any): void {
    super.getComponentJSONModel(RacesTableController.RACE_MODEL).setData(item);
    super.getEventBus()?.publish("race", "itemChanged", {});
  }

  /**
   * Builds an OR search across race number, labels, and comment.
   *
   * @param query - Trimmed search text applied to each race search field.
   * @returns One compound OR filter combining all supported race search fields.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("number", FilterOperator.Contains, query),
          new Filter("shortLabel", FilterOperator.Contains, query),
          new Filter("longLabel", FilterOperator.Contains, query),
          new Filter("comment", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Adds backend-provided filter categories only when they contain multiple choices; logs loading errors.
   *
   * @returns A promise resolving after dynamic categories are added, or after a loading failure is logged.
   */
  private async initFilterDialog(): Promise<void> {
    try {
      const filters: any = await super.getFilters();
      const viewSettingsDialog: ViewSettingsDialog = await super.getViewSettingsDialog(
        RacesTableController.FILTER_DIALOG
      );
      if (filters.boatClasses && filters.boatClasses.length > 1) {
        const boatClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "boatClass",
          text: "{i18n>common.boatClass}"
        });
        filters.boatClasses.forEach((boatClass: any) => {
          boatClassFilter.addItem(
            new ViewSettingsItem({
              text: `${boatClass.caption} (${boatClass.abbreviation})`,
              key: `boatClass/id___EQ___${boatClass.id}`
            })
          );
        });
        viewSettingsDialog.insertFilterItem(boatClassFilter, 0);
      }
      if (filters.ageClasses && filters.ageClasses.length > 1) {
        const ageClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "ageClass",
          text: "{i18n>common.ageClass}"
        });
        filters.ageClasses.forEach((ageClass: any) => {
          ageClassFilter.addItem(
            new ViewSettingsItem({
              text: `${ageClass.caption} ${ageClass.suffix}`,
              key: `ageClass/id___EQ___${ageClass.id}`
            })
          );
        });
        viewSettingsDialog.insertFilterItem(ageClassFilter, 1);
      }
      if (filters.distances && filters.distances.length > 1) {
        const distancesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "distance",
          text: "{i18n>common.distance}"
        });
        filters.distances.forEach((distance: any) => {
          distancesFilter.addItem(new ViewSettingsItem({ text: distance + "m", key: `distance___EQ___${distance}` }));
        });
        viewSettingsDialog.insertFilterItem(distancesFilter, 2);
      }
      if (filters.lightweight && filters.lightweight.length > 1) {
        const lightweightFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: false,
          key: "lightweight",
          text: "{i18n>common.lightweight}"
        });
        filters.lightweight.forEach((lightweight: any) => {
          const text: string = lightweight ? this.i18n("common.yes") : this.i18n("common.no");
          lightweightFilter.addItem(new ViewSettingsItem({ text: text, key: `lightweight___EQ___${lightweight}` }));
        });
        viewSettingsDialog.insertFilterItem(lightweightFilter, 5);
      }
    } catch (error) {
      console.error("Failed to initialize race filter dialog:", error);
    }
  }

  /**
   * Loads races for the active regatta into the view's list model.
   *
   * @returns Whether the list request succeeded; active-regatta loading failures still reject.
   */
  private async loadRacesModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url: string = `/api/regattas/${regatta.id}/races`;
    const racesModel: JSONModel = super.getViewJSONModel(RacesTableController.RACES_MODEL);
    return await super.updateJSONModel(racesModel, url);
  }
}
