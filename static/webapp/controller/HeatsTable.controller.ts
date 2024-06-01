import Table from "sap/m/Table";
import Formatter from "../model/Formatter";
import BaseTableController from "./BaseTable.controller";
import JSONModel from "sap/ui/model/json/JSONModel";
import ViewSettingsFilterItem from "sap/m/ViewSettingsFilterItem";
import ViewSettingsItem from "sap/m/ViewSettingsItem";
import { ListBase$SelectionChangeEvent } from "sap/m/ListBase";
import Button, { Button$PressEvent } from "sap/m/Button";
import FilterOperator from "sap/ui/model/FilterOperator";
import Filter from "sap/ui/model/Filter";
import { SearchField$LiveChangeEvent } from "sap/m/SearchField";
import ListItemBase from "sap/m/ListItemBase";
import ViewSettingsDialog from "sap/m/ViewSettingsDialog";
import Context from "sap/ui/model/Context";
import { Route$MatchedEvent } from "sap/ui/core/routing/Route";
import MessageToast from "sap/m/MessageToast";

/**
 * @namespace de.regatta_hd.infoportal.controller
 */
export default class HeatsTableController extends BaseTableController {

  formatter: Formatter = Formatter;
  private readonly heatsModel: JSONModel = new JSONModel();

  onInit(): void {
    super.init(super.getView()?.byId("heatsTable") as Table, "heat" /* eventBus channel */);

    super.getView()?.addStyleClass(super.getContentDensityClass());
    super.setViewModel(this.heatsModel, "heats");
    super.getRouter()?.getRoute("heats")?.attachMatched(async (_: Route$MatchedEvent) => await this.loadHeatsModel(), this);

    const filters: any = (super.getComponentModel("filters") as JSONModel).getData();
    super.getViewSettingsDialog("de.regatta_hd.infoportal.view.HeatsFilterDialog").then((viewSettingsDialog: ViewSettingsDialog) => {
      if (filters.dates && filters.dates.length > 1) {
        const datesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "day", text: "{i18n>common.day}" });
        filters.dates.forEach((date: any) => {
          datesFilter.addItem(new ViewSettingsItem({ text: Formatter.weekDayDateLabel(date), key: "dateTime___Contains___" + date }));
        });
        viewSettingsDialog.insertFilterItem(datesFilter, 0);
      }
      if (filters.blocks && filters.blocks.length > 1) {
        const blocksFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "block", text: "Block" });
        filters.blocks.forEach((block: any) => {
          blocksFilter.addItem(new ViewSettingsItem({
            text: `${Formatter.dayTimeIsoLabel(block.begin)} - ${Formatter.dayTimeIsoLabel(block.end)}: ${block.heats} {i18n>common.heats}`,
            key: `dateTime___BT___${block.begin}___${block.end}`
          }));
        });
        viewSettingsDialog.insertFilterItem(blocksFilter, 1);
      }
      if (filters.boatClasses && filters.boatClasses.length > 1) {
        const boatClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "boatClass", text: "{i18n>common.boatClass}" });
        filters.boatClasses.forEach((boatClass: any) => {
          boatClassFilter.addItem(new ViewSettingsItem({
            text: `${boatClass.caption} (${boatClass.abbreviation})`, key: `race/boatClass/id___EQ___${boatClass.id}`
          }));
        });
        viewSettingsDialog.insertFilterItem(boatClassFilter, 2);
      }
      if (filters.ageClasses && filters.ageClasses.length > 1) {
        const ageClassFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "ageClass", text: "{i18n>common.ageClass}" });
        filters.ageClasses.forEach((ageClass: any) => {
          ageClassFilter.addItem(new ViewSettingsItem({ text: `${ageClass.caption} ${ageClass.suffix}`, key: `race/ageClass/id___EQ___${ageClass.id}` }));
        });
        viewSettingsDialog.insertFilterItem(ageClassFilter, 3);
      }
      if (filters.distances && filters.distances.length > 1) {
        const distancesFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "distance", text: "{i18n>common.distance}" });
        filters.distances.forEach((distance: any) => {
          distancesFilter.addItem(new ViewSettingsItem({ text: distance + "m", key: "race/distance___EQ___" + distance }));
        });
        viewSettingsDialog.insertFilterItem(distancesFilter, 5);
      }
      if (filters.rounds && filters.rounds.length > 1) {
        const roundFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: true, key: "round", text: "{i18n>common.round}" });
        filters.rounds.forEach((round: any) => {
          roundFilter.addItem(new ViewSettingsItem({ text: Formatter.roundLabel(round.code), key: "roundCode___EQ___" + round.code }))
        });
        viewSettingsDialog.insertFilterItem(roundFilter, 6);
      }
      if (filters.lightweight && filters.lightweight.length > 1) {
        const lightweightFilter: ViewSettingsFilterItem = new ViewSettingsFilterItem({ multiSelect: false, key: "lightweight", text: "{i18n>common.lightweight}" });
        filters.lightweight.forEach((lightweight: any) => {
          const text: string = lightweight ? this.i18n("common.yes") : this.i18n("common.no");
          lightweightFilter.addItem(new ViewSettingsItem({ text: text, key: "race/lightweight___EQ___" + lightweight }));
        });
        viewSettingsDialog.insertFilterItem(lightweightFilter, 7);
      }
    });
  }

  onSelectionChange(event: ListBase$SelectionChangeEvent): void {
    const selectedItem: ListItemBase | undefined = event.getParameter("listItem");
    if (selectedItem) {
      const bindingCtx: Context | null | undefined = selectedItem.getBindingContext("heats");
      const heat: any = bindingCtx?.getModel().getProperty(bindingCtx.getPath());

      const index: number = this.table.indexOfItem(selectedItem);
      const count = this.table.getItems().length;
      // store navigation meta information in selected item
      heat._nav = { isFirst: index == 0, isLast: index == count - 1 };

      this.onItemChanged(heat);
      super.displayTarget("heatRegistrations");
    }
  }

  onNavBack(): void {
    super.navBack("startpage");
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
    this.loadHeatsModel().then((updated: boolean) => {
      if (updated) {
        MessageToast.show(this.i18n("msg.dataUpdated"));
      }
    }).finally(() => source.setEnabled(true));
  }

  onItemChanged(item: any): void {
    (super.getComponentModel("heat") as JSONModel).setData(item);
    super.getEventBus()?.publish("heat", "itemChanged", {});
  }

  async onSortButtonPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.HeatsSortDialog")).open();
  }

  async onFilterButtonPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.HeatsFilterDialog")).open();
  }

  async onClearFilterPress(event: Button$PressEvent): Promise<void> {
    (await super.getViewSettingsDialog("de.regatta_hd.infoportal.view.HeatsFilterDialog")).clearFilters();
    super.clearFilters();
    super.applyFilters();
  }

  private createSearchFilters(query: string): Filter[] {
    return [new Filter({
      filters: [
        new Filter("race/number", FilterOperator.Contains, query),
        new Filter("race/shortLabel", FilterOperator.Contains, query),
        new Filter("race/longLabel", FilterOperator.Contains, query),
        new Filter("race/comment", FilterOperator.Contains, query)
      ],
      and: false
    })]
  }

  private async loadHeatsModel(): Promise<boolean> {
    return await super.updateJSONModel(this.heatsModel, `/api/regattas/${super.getRegattaId()}/heats`, this.table);
  }

}