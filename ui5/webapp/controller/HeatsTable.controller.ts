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
import { NavigationData } from "../model/Types";
import BaseTableController from "./BaseTable.controller";

/**
 * Heat list with schedule-based sorting, search, dynamic filters, and selection shared with heat details.
 *
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatsTableController extends BaseTableController {
  private static readonly FILTER_DIALOG: string = "de.regatta_hd.infoportal.view.HeatsFilterDialog";
  private static readonly SORT_DIALOG: string = "de.regatta_hd.infoportal.view.HeatsSortDialog";
  /** Component model containing the heat selected for the detail view. */
  static readonly HEAT_MODEL: string = "heat";
  /** Component model containing list-navigation state, kept separate from heat data. */
  static readonly HEAT_NAV_MODEL: string = "heatNav";
  private static readonly HEATS_MODEL: string = "heats";

  readonly formatter: Formatter = Formatter;

  /** Configures table navigation, route-triggered loading, chronological sorting, and dynamic filter options. */
  onInit(): void {
    super.init(
      super.getView()?.byId("heatsTable") as Table,
      "heat" /* eventBus channel */,
      HeatsTableController.HEAT_NAV_MODEL
    );

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(new JSONModel(), HeatsTableController.HEATS_MODEL);
    super
      .getRouter()
      ?.getRoute(Routes.Heats)
      ?.attachMatched((_: Route$MatchedEvent) => {
        super.runAsync(this.loadHeatsModel(), "Failed to load heats:");
      }, this);

    super.sortTable("heatDateTime", false, "dateTime");

    void this.initFilterDialog();
  }

  /**
   * Stores the selected heat and its list position before opening details.
   *
   * @param event - Selection event whose list item is bound to the heat to display.
   */
  onItemPress(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("heats");
      const heat: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      const index: number = this.table.indexOfItem(selectedItem);
      const count = this.table.getItems().length;
      const navData: NavigationData = {
        isFirst: index === 0,
        isLast: index === count - 1,
        disabled: false,
        back: undefined
      };
      super.updateNavModel(navData);

      this.onItemChanged(heat);
      super.navToHeatDetails(heat.id);
    }
  }

  /** Returns to the launchpad and reduces the growing threshold for the next visit. */
  onNavBack(): void {
    super.navToStartPage();
    // reduce table growing threshold to improve performance next time table is shown
    this.table.setGrowingThreshold(30);
  }

  /**
   * Applies trimmed race-related text search together with the active heat-dialog filters.
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
   * Reloads heats and restores the refresh button on both success and failure.
   *
   * @param event - Press event providing the refresh button to disable during loading.
   * @returns A promise resolving after loading, feedback, and button restoration, including handled failures.
   */
  async onRefreshButtonPress(event: Button$PressEvent): Promise<void> {
    const source: Button = event.getSource();
    source.setEnabled(false);
    try {
      const succeeded: boolean = await this.loadHeatsModel();
      super.showDataUpdatedMessage(succeeded);
    } catch (error) {
      console.error("Failed to refresh heats:", error);
      super.showDataUpdatedMessage(false);
    } finally {
      source.setEnabled(true);
    }
  }

  /**
   * Updates the shared heat model and notifies the visible detail view through the `heat` event-bus channel.
   *
   * @param item - Selected heat data to place in the component's shared heat model.
   */
  protected onItemChanged(item: any): void {
    super.getComponentJSONModel(HeatsTableController.HEAT_MODEL).setData(item);
    super.getEventBus()?.publish("heat", "itemChanged", {});
  }

  /**
   * Opens the heat sorting dialog, logging fragment-loading failures.
   *
   * @param _event - Sort-button press event; unused because the dialog is fixed for this controller.
   * @returns A promise resolving after the dialog opens or a loading failure is logged.
   */
  async onSortButtonPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog(HeatsTableController.SORT_DIALOG);
      dialog.open();
    } catch (error) {
      console.error("Failed to load sort dialog:", error);
    }
  }

  /**
   * Opens the heat filtering dialog, logging fragment-loading failures.
   *
   * @param _event - Filter-button press event; unused because the dialog is fixed for this controller.
   * @returns A promise resolving after the dialog opens or a loading failure is logged.
   */
  async onFilterButtonPress(_event: Button$PressEvent): Promise<void> {
    try {
      const dialog = await super.getViewSettingsDialog(HeatsTableController.FILTER_DIALOG);
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
      const dialog = await super.getViewSettingsDialog(HeatsTableController.FILTER_DIALOG);
      dialog.clearFilters();
    } catch (error) {
      console.error("Failed to load filter dialog:", error);
    }
    super.clearFilters();
    super.applyFilters();
  }

  /**
   * Builds an OR search across the heat's race number, labels, and comment.
   *
   * @param query - Trimmed search text applied to each race search field.
   * @returns One compound OR filter combining all supported race search fields for the heat list.
   */
  private createSearchFilters(query: string): Filter[] {
    return [
      new Filter({
        filters: [
          new Filter("race/number", FilterOperator.Contains, query),
          new Filter("race/shortLabel", FilterOperator.Contains, query),
          new Filter("race/longLabel", FilterOperator.Contains, query),
          new Filter("race/comment", FilterOperator.Contains, query)
        ],
        and: false
      })
    ];
  }

  /**
   * Adds multi-choice date, block, and race filter categories using keys decoded by the base table controller.
   *
   * @returns A promise resolving after dynamic categories are added, or after a loading failure is logged.
   */
  private async initFilterDialog(): Promise<void> {
    try {
      const filters: any = await super.getFilters();
      const viewSettingsDialog: ViewSettingsDialog = await super.getViewSettingsDialog(
        HeatsTableController.FILTER_DIALOG
      );
      if (filters.dates && filters.dates.length > 1) {
        const datesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "day",
          text: "{i18n>common.day}"
        });
        filters.dates.forEach((date: any) => {
          datesFilter.addItem(
            new ViewSettingsItem({ text: Formatter.weekDayDateLabel(date), key: `dateTime___Contains___${date}` })
          );
        });
        viewSettingsDialog.insertFilterItem(datesFilter, 0);
      }
      if (filters.blocks && filters.blocks.length > 1) {
        const blocksFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "block",
          text: "Block"
        });
        filters.blocks.forEach((block: any) => {
          blocksFilter.addItem(
            new ViewSettingsItem({
              text: `${Formatter.dayTimeIsoLabel(block.begin)} - ${Formatter.dayTimeIsoLabel(block.end)}: ${block.heats} {i18n>common.heats}`,
              key: `dateTime___BT___${block.begin}___${block.end}`
            })
          );
        });
        viewSettingsDialog.insertFilterItem(blocksFilter, 1);
      }
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
              key: `race/boatClass/id___EQ___${boatClass.id}`
            })
          );
        });
        viewSettingsDialog.insertFilterItem(boatClassFilter, 2);
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
              key: `race/ageClass/id___EQ___${ageClass.id}`
            })
          );
        });
        viewSettingsDialog.insertFilterItem(ageClassFilter, 3);
      }
      if (filters.distances && filters.distances.length > 1) {
        const distancesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "distance",
          text: "{i18n>common.distance}"
        });
        filters.distances.forEach((distance: any) => {
          distancesFilter.addItem(
            new ViewSettingsItem({ text: distance + "m", key: `race/distance___EQ___${distance}` })
          );
        });
        viewSettingsDialog.insertFilterItem(distancesFilter, 5);
      }
      if (filters.rounds && filters.rounds.length > 1) {
        const roundFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: true,
          key: "round",
          text: "{i18n>common.round}"
        });
        filters.rounds.forEach((round: any) => {
          roundFilter.addItem(
            new ViewSettingsItem({ text: Formatter.roundLabel(round.code), key: `roundCode___EQ___${round.code}` })
          );
        });
        viewSettingsDialog.insertFilterItem(roundFilter, 6);
      }
      if (filters.lightweight && filters.lightweight.length > 1) {
        const lightweightFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({
          multiSelect: false,
          key: "lightweight",
          text: "{i18n>common.lightweight}"
        });
        filters.lightweight.forEach((lightweight: any) => {
          const text: string = lightweight ? this.i18n("common.yes") : this.i18n("common.no");
          lightweightFilter.addItem(
            new ViewSettingsItem({ text: text, key: `race/lightweight___EQ___${lightweight}` })
          );
        });
        viewSettingsDialog.insertFilterItem(lightweightFilter, 7);
      }
    } catch (error) {
      console.error("Failed to initialize heat filter dialog:", error);
    }
  }

  /**
   * Loads heats for the active regatta into the view's list model.
   *
   * @returns Whether the list request succeeded; active-regatta loading failures still reject.
   */
  private async loadHeatsModel(): Promise<boolean> {
    const regatta: any = await super.getActiveRegatta();
    const url: string = `/api/regattas/${regatta.id}/heats`;
    const heatsModel: JSONModel = super.getViewJSONModel(HeatsTableController.HEATS_MODEL);
    return await super.updateJSONModel(heatsModel, url);
  }
}
